use std::sync::mpsc::{channel, Receiver, Sender};

use crate::mosaic::SquareMosaic;
use crate::set::builder::{SetBuilder, ShardBuilder};
use crate::set::square::SquareMosaicSet;

pub struct InMemorySquareMosaicSetBuilder<const N: usize, M: SquareMosaic<N> + Send> {
    tx: Sender<SquareMosaicSet<N, M>>,
    rx: Receiver<SquareMosaicSet<N, M>>,
}

impl<const N: usize, M: SquareMosaic<N> + Send> InMemorySquareMosaicSetBuilder<N, M> {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        Self { tx, rx }
    }
}

impl<const N: usize, M: SquareMosaic<N> + Send> SetBuilder
    for InMemorySquareMosaicSetBuilder<N, M>
{
    type Item = M;
    type Shard = InMemorySquareMosaicShardBuilder<N, M>;
    type Result = SquareMosaicSet<N, M>;

    fn new_shard(&mut self) -> Self::Shard {
        InMemorySquareMosaicShardBuilder {
            tx: self.tx.clone(),
            set: Some(SquareMosaicSet::new()),
        }
    }

    fn finish(self) -> Self::Result {
        drop(self.tx);
        let mut result = SquareMosaicSet::new();
        while let Ok(shard_result) = self.rx.recv() {
            result.extend(shard_result);
        }
        result
    }
}

pub struct InMemorySquareMosaicShardBuilder<const N: usize, M: SquareMosaic<N> + Send> {
    tx: Sender<SquareMosaicSet<N, M>>,
    set: Option<SquareMosaicSet<N, M>>,
}

impl<const N: usize, M: SquareMosaic<N> + Send> ShardBuilder
    for InMemorySquareMosaicShardBuilder<N, M>
{
    type Item = M;

    fn insert(&mut self, item: M) {
        self.set.as_mut().unwrap().insert(item);
    }

    fn finish(self) {}
}

impl<const N: usize, M: SquareMosaic<N> + Send> Clone for InMemorySquareMosaicShardBuilder<N, M> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
            set: Some(SquareMosaicSet::new()),
        }
    }
}

impl<const N: usize, M: SquareMosaic<N> + Send> Drop for InMemorySquareMosaicShardBuilder<N, M> {
    fn drop(&mut self) {
        if let Some(set) = self.set.take() {
            self.tx.send(set).unwrap();
        }
    }
}
