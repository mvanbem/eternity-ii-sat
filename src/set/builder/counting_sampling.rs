use std::sync::mpsc::{channel, Receiver, Sender};

use crate::set::builder::{SetBuilder, ShardBuilder};

pub struct CountingSamplingSetBuilder<T> {
    tx: Sender<(usize, Option<T>)>,
    rx: Receiver<(usize, Option<T>)>,
}

impl<T> CountingSamplingSetBuilder<T> {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        Self { tx, rx }
    }
}

impl<T: Send> SetBuilder for CountingSamplingSetBuilder<T> {
    type Item = T;
    type Shard = CountingSamplingShardBuilder<T>;
    type Result = (usize, Option<T>);

    fn new_shard(&mut self) -> CountingSamplingShardBuilder<T> {
        CountingSamplingShardBuilder {
            tx: Some(self.tx.clone()),
            count: 0,
            sample: None,
        }
    }

    fn finish(self) -> Self::Result {
        drop(self.tx);
        let mut count = 0;
        let mut value = None;
        while let Ok((shard_count, shard_value)) = self.rx.recv() {
            count += shard_count;
            if value.is_none() {
                value = shard_value;
            }
        }
        (count, value)
    }
}

pub struct CountingSamplingShardBuilder<T: Send> {
    tx: Option<Sender<(usize, Option<T>)>>,
    count: usize,
    sample: Option<T>,
}

impl<T: Send> ShardBuilder for CountingSamplingShardBuilder<T> {
    type Item = T;

    fn insert(&mut self, item: Self::Item) {
        self.count += 1;
        if self.sample.is_none() {
            self.sample = Some(item);
        }
    }

    fn finish(self) {}
}

impl<T: Send> Clone for CountingSamplingShardBuilder<T> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
            count: 0,
            sample: None,
        }
    }
}

impl<T: Send> Drop for CountingSamplingShardBuilder<T> {
    fn drop(&mut self) {
        if let Some(tx) = self.tx.take() {
            tx.send((self.count, self.sample.take())).unwrap();
        }
    }
}
