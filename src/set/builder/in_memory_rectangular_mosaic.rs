use std::sync::mpsc::{channel, Receiver, Sender};

use crate::mosaic::RectangularMosaic;
use crate::set::builder::{SetBuilder, ShardBuilder};
use crate::set::rectangle::RectangularMosaicSet;

pub struct InMemoryRectangularMosaicSetBuilder<
    const W: usize,
    const H: usize,
    M: RectangularMosaic<W, H>,
> {
    tx: Sender<RectangularMosaicSet<W, H, M>>,
    rx: Receiver<RectangularMosaicSet<W, H, M>>,
}

impl<const W: usize, const H: usize, M: RectangularMosaic<W, H>>
    InMemoryRectangularMosaicSetBuilder<W, H, M>
{
    pub fn new() -> Self {
        let (tx, rx) = channel();
        Self { tx, rx }
    }
}

impl<const W: usize, const H: usize, M: RectangularMosaic<W, H> + Send> SetBuilder
    for InMemoryRectangularMosaicSetBuilder<W, H, M>
{
    type Item = M;
    type Shard = InMemoryRectangularMosaicShardBuilder<W, H, M>;
    type Result = RectangularMosaicSet<W, H, M>;

    fn new_shard(&mut self) -> Self::Shard {
        InMemoryRectangularMosaicShardBuilder {
            tx: self.tx.clone(),
            set: Some(RectangularMosaicSet::new()),
        }
    }

    fn finish(self) -> Self::Result {
        drop(self.tx);
        let mut result = RectangularMosaicSet::new();
        while let Ok(shard_result) = self.rx.recv() {
            result.extend(shard_result);
        }
        result
    }
}

pub struct InMemoryRectangularMosaicShardBuilder<
    const W: usize,
    const H: usize,
    M: RectangularMosaic<W, H> + Send,
> {
    tx: Sender<RectangularMosaicSet<W, H, M>>,
    set: Option<RectangularMosaicSet<W, H, M>>,
}

impl<const W: usize, const H: usize, M: RectangularMosaic<W, H> + Send> ShardBuilder
    for InMemoryRectangularMosaicShardBuilder<W, H, M>
{
    type Item = M;

    fn insert(&mut self, item: M) {
        self.set.as_mut().unwrap().insert(item);
    }

    fn finish(self) {}
}

impl<const W: usize, const H: usize, M: RectangularMosaic<W, H> + Send> Clone
    for InMemoryRectangularMosaicShardBuilder<W, H, M>
{
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
            set: Some(RectangularMosaicSet::new()),
        }
    }
}

impl<const W: usize, const H: usize, M: RectangularMosaic<W, H> + Send> Drop
    for InMemoryRectangularMosaicShardBuilder<W, H, M>
{
    fn drop(&mut self) {
        if let Some(set) = self.set.take() {
            self.tx.send(set).unwrap();
        }
    }
}
