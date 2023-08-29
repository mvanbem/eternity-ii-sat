use std::marker::PhantomData;
use std::sync::mpsc::{channel, Receiver, Sender};

use crate::set::builder::{SetBuilder, ShardBuilder};

pub struct CountingSetBuilder<T> {
    tx: Sender<usize>,
    rx: Receiver<usize>,
    _phantom_t: PhantomData<fn(T)>,
}

impl<T> CountingSetBuilder<T> {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        Self {
            tx,
            rx,
            _phantom_t: PhantomData,
        }
    }
}

impl<T> SetBuilder for CountingSetBuilder<T> {
    type Item = T;
    type Shard = CountingShardBuilder<T>;
    type Result = usize;

    fn new_shard(&mut self) -> CountingShardBuilder<T> {
        CountingShardBuilder {
            tx: Some(self.tx.clone()),
            count: 0,
            _phantom_t: PhantomData,
        }
    }

    fn finish(self) -> Self::Result {
        drop(self.tx);
        let mut count = 0;
        while let Ok(shard_count) = self.rx.recv() {
            count += shard_count;
        }
        count
    }
}

pub struct CountingShardBuilder<T> {
    tx: Option<Sender<usize>>,
    count: usize,
    _phantom_t: PhantomData<fn(T)>,
}

impl<T> ShardBuilder for CountingShardBuilder<T> {
    type Item = T;

    fn insert(&mut self, _item: Self::Item) {
        self.count += 1;
    }

    fn finish(self) {}
}

impl<T> Clone for CountingShardBuilder<T> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
            count: 0,
            _phantom_t: PhantomData,
        }
    }
}

impl<T> Drop for CountingShardBuilder<T> {
    fn drop(&mut self) {
        if let Some(tx) = self.tx.take() {
            tx.send(self.count).unwrap();
        }
    }
}
