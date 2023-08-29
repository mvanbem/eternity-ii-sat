use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Debug, Formatter};

use strum::IntoEnumIterator;

use crate::edge::ArrayEdge;
use crate::mosaic::{RotatedSquareMosaic, SquareMosaic};
use crate::{Rotation, Side};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct RotatedSquareMosaicIndex {
    index: usize,
    rotation: Rotation,
}

impl Debug for RotatedSquareMosaicIndex {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "({}, {:?})", self.index, self.rotation)
    }
}

#[derive(Clone, Debug)]
pub struct SquareMosaicSet<const N: usize, M: SquareMosaic<N>> {
    mosaics: Vec<M>,
    index_by_rotated_right_edge: BTreeMap<ArrayEdge<N>, BTreeSet<RotatedSquareMosaicIndex>>,
}

impl<const N: usize, M: SquareMosaic<N>> SquareMosaicSet<N, M> {
    pub fn new() -> Self {
        Self {
            mosaics: Vec::new(),
            index_by_rotated_right_edge: BTreeMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.mosaics.len()
    }

    pub fn iter_mosaics(&self) -> impl Iterator<Item = &M> + '_ {
        self.mosaics.iter()
    }

    fn get(&self, i: RotatedSquareMosaicIndex) -> RotatedSquareMosaic<N, M> {
        RotatedSquareMosaic {
            mosaic: &self.mosaics[i.index],
            rotation: i.rotation,
        }
    }

    pub fn assert_distinct(&self) {
        let all_rotations = BTreeSet::from_iter(
            Rotation::iter()
                .map(|rotation| {
                    self.mosaics
                        .iter()
                        .map(move |mosaic| mosaic.with_square_rotation(rotation))
                })
                .flatten(),
        );
        // If any two rotations of a mosaic appear, the set will deduplicate
        // them and the sizes won't match.
        assert_eq!(all_rotations.len(), 4 * self.len());
    }

    pub fn insert(&mut self, mosaic: M) {
        let index = self.mosaics.len();

        for rotation in Rotation::iter() {
            self.index_by_rotated_right_edge
                .entry(mosaic.with_square_rotation(rotation).edge(Side::Right))
                .or_default()
                .insert(RotatedSquareMosaicIndex { index, rotation });
        }

        self.mosaics.push(mosaic);
    }

    pub fn extend(&mut self, mut other: Self) {
        let base_index = self.mosaics.len();
        self.mosaics.extend(other.mosaics.drain(..));
        for (edge, mosaics) in other.index_by_rotated_right_edge {
            let entry = self.index_by_rotated_right_edge.entry(edge).or_default();
            for i in mosaics {
                entry.insert(RotatedSquareMosaicIndex {
                    index: i.index + base_index,
                    rotation: i.rotation,
                });
            }
        }
    }

    pub fn iter_by_edge(
        &self,
        side: Side,
    ) -> impl Iterator<
        Item = (
            &ArrayEdge<N>,
            impl Iterator<Item = (usize, RotatedSquareMosaic<N, M>)> + '_,
        ),
    > + '_ {
        self.index_by_rotated_right_edge
            .iter()
            .map(move |(edge, mosaics)| {
                (
                    edge,
                    mosaics
                        .iter()
                        .map(move |&i| (i.index, self.get(i) + side.rotation_from_right())),
                )
            })
    }

    pub fn query(
        &self,
        side: Side,
        edge: &ArrayEdge<N>,
    ) -> impl Iterator<Item = (usize, RotatedSquareMosaic<N, M>)> + '_ {
        self.index_by_rotated_right_edge
            .get(edge)
            .into_iter()
            .flatten()
            .map(move |&i| (i.index, self.get(i) + side.rotation_from_right()))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use crate::edge::ArrayEdge;
    use crate::{Rotation, Side};

    use super::SquareMosaicSet;

    #[test]
    fn iter_by_edge() {
        let mosaic = mosaic![[0, 1], [16, 17]];
        let edge_aa = ArrayEdge::from_byte_string(b"aa");
        let edge_fi = ArrayEdge::from_byte_string(b"fi");
        let edge_or = ArrayEdge::from_byte_string(b"or");
        let identity = &mosaic + Rotation::Identity;
        let quarter_turn_left = &mosaic + Rotation::QuarterTurnLeft;
        let half_turn = &mosaic + Rotation::HalfTurn;
        let quarter_turn_right = &mosaic + Rotation::QuarterTurnRight;

        let mut set = SquareMosaicSet::new();
        set.insert(mosaic);

        let iter_by_edge_as_btree = |side| {
            BTreeMap::from_iter(
                set.iter_by_edge(side)
                    .map(|(edge, mosaics)| (*edge, BTreeSet::from_iter(mosaics))),
            )
        };
        assert_eq!(
            iter_by_edge_as_btree(Side::Right),
            btree_map![
                edge_aa => btree_set![
                    (0, half_turn),
                    (0, quarter_turn_right),
                ],
                edge_fi => btree_set![
                    (0, identity),
                ],
                edge_or => btree_set![
                    (0, quarter_turn_left),
                ],
            ],
        );
        assert_eq!(
            iter_by_edge_as_btree(Side::Top),
            btree_map![
                edge_aa => btree_set![
                    (0, identity),
                    (0, quarter_turn_right),
                ],
                edge_fi => btree_set![
                    (0, quarter_turn_left),
                ],
                edge_or => btree_set![
                    (0, half_turn),
                ],
            ],
        );
        assert_eq!(
            iter_by_edge_as_btree(Side::Left),
            btree_map![
                edge_aa => btree_set![
                    (0, identity),
                    (0, quarter_turn_left),
                ],
                edge_fi => btree_set![
                    (0, half_turn),
                ],
                edge_or => btree_set![
                    (0, quarter_turn_right),
                ],
            ],
        );
        assert_eq!(
            iter_by_edge_as_btree(Side::Bottom),
            btree_map![
                edge_aa => btree_set![
                    (0, quarter_turn_left),
                    (0, half_turn),
                ],
                edge_fi => btree_set![
                    (0, quarter_turn_right),
                ],
                edge_or => btree_set![
                    (0, identity),
                ],
            ],
        );

        assert_eq!(
            BTreeSet::from_iter(set.query(Side::Right, &edge_aa)),
            btree_set![(0, half_turn), (0, quarter_turn_right)],
        );
        assert_eq!(
            BTreeSet::from_iter(set.query(Side::Right, &edge_fi)),
            btree_set![(0, identity)],
        );
        assert_eq!(
            BTreeSet::from_iter(set.query(Side::Right, &edge_or)),
            btree_set![(0, quarter_turn_left)],
        );
        assert_eq!(
            BTreeSet::from_iter(set.query(Side::Top, &edge_aa)),
            btree_set![(0, identity), (0, quarter_turn_right)],
        );
        assert_eq!(
            BTreeSet::from_iter(set.query(Side::Top, &edge_fi)),
            btree_set![(0, quarter_turn_left)],
        );
        assert_eq!(
            BTreeSet::from_iter(set.query(Side::Top, &edge_or)),
            btree_set![(0, half_turn)],
        );
        assert_eq!(
            BTreeSet::from_iter(set.query(Side::Left, &edge_aa)),
            btree_set![(0, identity), (0, quarter_turn_left)],
        );
        assert_eq!(
            BTreeSet::from_iter(set.query(Side::Left, &edge_fi)),
            btree_set![(0, half_turn)],
        );
        assert_eq!(
            BTreeSet::from_iter(set.query(Side::Left, &edge_or)),
            btree_set![(0, quarter_turn_right)],
        );
        assert_eq!(
            BTreeSet::from_iter(set.query(Side::Bottom, &edge_aa)),
            btree_set![(0, quarter_turn_left), (0, half_turn)],
        );
        assert_eq!(
            BTreeSet::from_iter(set.query(Side::Bottom, &edge_fi)),
            btree_set![(0, quarter_turn_right)],
        );
        assert_eq!(
            BTreeSet::from_iter(set.query(Side::Bottom, &edge_or)),
            btree_set![(0, identity)],
        );
    }
}
