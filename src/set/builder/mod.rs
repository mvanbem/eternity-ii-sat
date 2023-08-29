pub mod counting;
pub mod counting_sampling;
pub mod in_memory_rectangular_mosaic;
pub mod in_memory_square_mosaic;

pub trait SetBuilder {
    type Item;
    type Shard: ShardBuilder<Item = Self::Item>;
    type Result;

    fn new_shard(&mut self) -> Self::Shard;
    fn finish(self) -> Self::Result;
}

pub trait ShardBuilder: Clone + Send {
    type Item;

    fn insert(&mut self, item: Self::Item);
    fn finish(self);
}
