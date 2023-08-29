use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Debug, Formatter};

use strum::IntoEnumIterator;

use crate::edge::ArrayEdge;
use crate::mosaic::{RectangularMosaic, RotatedRectangularMosaic};
use crate::rectangular::{HorizontalSide, RectangularRotation, VerticalSide};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RotatedRectangularMosaicIndex {
    index: usize,
    rotation: RectangularRotation,
}

impl Debug for RotatedRectangularMosaicIndex {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "({}, {:?})", self.index, self.rotation)
    }
}

#[derive(Clone, Debug)]
pub struct RectangularMosaicSet<const W: usize, const H: usize, M: RectangularMosaic<W, H>> {
    mosaics: Vec<M>,
    index_by_rotated_right_edge: BTreeMap<ArrayEdge<H>, BTreeSet<RotatedRectangularMosaicIndex>>,
    index_by_rotated_top_edge: BTreeMap<ArrayEdge<W>, BTreeSet<RotatedRectangularMosaicIndex>>,
}

impl<const W: usize, const H: usize, M: RectangularMosaic<W, H>> RectangularMosaicSet<W, H, M> {
    pub fn new() -> Self {
        Self {
            mosaics: Vec::new(),
            index_by_rotated_right_edge: BTreeMap::new(),
            index_by_rotated_top_edge: BTreeMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.mosaics.len()
    }

    pub fn iter_mosaics(&self) -> impl Iterator<Item = &M> + '_ {
        self.mosaics.iter()
    }

    fn get(&self, i: RotatedRectangularMosaicIndex) -> RotatedRectangularMosaic<W, H, M> {
        RotatedRectangularMosaic {
            mosaic: &self.mosaics[i.index],
            rotation: i.rotation,
        }
    }

    pub fn assert_distinct(&self) {
        let all_rotations = BTreeSet::from_iter(
            RectangularRotation::iter()
                .map(|rotation| {
                    self.mosaics
                        .iter()
                        .map(move |mosaic| mosaic.with_rectangular_rotation(rotation))
                })
                .flatten(),
        );
        // If both rotations of a mosaic appear, the set will deduplicate them
        // and the sizes won't match.
        assert_eq!(all_rotations.len(), 2 * self.len());
    }

    pub fn insert(&mut self, mosaic: M) {
        let index = self.mosaics.len();

        for rotation in RectangularRotation::iter() {
            self.index_by_rotated_right_edge
                .entry(
                    (RotatedRectangularMosaic::from(&mosaic) + rotation)
                        .vertical_edge(VerticalSide::Right),
                )
                .or_default()
                .insert(RotatedRectangularMosaicIndex { index, rotation });

            self.index_by_rotated_top_edge
                .entry(
                    (RotatedRectangularMosaic::from(&mosaic) + rotation)
                        .horizontal_edge(HorizontalSide::Top),
                )
                .or_default()
                .insert(RotatedRectangularMosaicIndex { index, rotation });
        }

        self.mosaics.push(mosaic);
    }

    pub fn extend(&mut self, mut other: Self) {
        let base_index = self.mosaics.len();
        self.mosaics.extend(other.mosaics.drain(..));
        for (edge, mosaics) in other.index_by_rotated_right_edge {
            let entry = self.index_by_rotated_right_edge.entry(edge).or_default();
            for i in mosaics {
                entry.insert(RotatedRectangularMosaicIndex {
                    index: i.index + base_index,
                    rotation: i.rotation,
                });
            }
        }
        for (edge, mosaics) in other.index_by_rotated_top_edge {
            let entry = self.index_by_rotated_top_edge.entry(edge).or_default();
            for i in mosaics {
                entry.insert(RotatedRectangularMosaicIndex {
                    index: i.index + base_index,
                    rotation: i.rotation,
                });
            }
        }
    }

    pub fn iter_by_vertical_edge(
        &self,
        side: VerticalSide,
    ) -> impl Iterator<
        Item = (
            &ArrayEdge<H>,
            impl Iterator<Item = (usize, RotatedRectangularMosaic<W, H, M>)> + '_,
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

    pub fn iter_by_horizontal_edge(
        &self,
        side: HorizontalSide,
    ) -> impl Iterator<
        Item = (
            &ArrayEdge<W>,
            impl Iterator<Item = (usize, RotatedRectangularMosaic<W, H, M>)> + '_,
        ),
    > + '_ {
        self.index_by_rotated_top_edge
            .iter()
            .map(move |(edge, mosaics)| {
                (
                    edge,
                    mosaics
                        .iter()
                        .map(move |&i| (i.index, self.get(i) + side.rotation_from_top())),
                )
            })
    }

    pub fn query_vertical(
        &self,
        side: VerticalSide,
        edge: &ArrayEdge<H>,
    ) -> impl Iterator<Item = (usize, RotatedRectangularMosaic<W, H, M>)> + '_ {
        self.index_by_rotated_right_edge
            .get(edge)
            .into_iter()
            .flatten()
            .map(move |&i| (i.index, self.get(i) + side.rotation_from_right()))
    }

    pub fn query_horizontal(
        &self,
        side: HorizontalSide,
        edge: &ArrayEdge<W>,
    ) -> impl Iterator<Item = (usize, RotatedRectangularMosaic<W, H, M>)> + '_ {
        self.index_by_rotated_top_edge
            .get(edge)
            .into_iter()
            .flatten()
            .map(move |&i| (i.index, self.get(i) + side.rotation_from_top()))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use crate::edge::ArrayEdge;
    use crate::rectangular::{HorizontalSide, RectangularRotation, VerticalSide};

    use super::RectangularMosaicSet;

    #[test]
    fn iter_by_edge() {
        let mosaic = mosaic![[0, 1, 2], [16, 17, 18]];
        let edge_bv = ArrayEdge::from_byte_string(b"bv");
        let edge_aa = ArrayEdge::from_byte_string(b"aa");
        let edge_aaa = ArrayEdge::from_byte_string(b"aaa");
        let edge_sor = ArrayEdge::from_byte_string(b"sor");
        let identity = &mosaic + RectangularRotation::Identity;
        let half_turn = &mosaic + RectangularRotation::HalfTurn;

        let mut set = RectangularMosaicSet::new();
        set.insert(mosaic);

        let iter_by_vertical_edge_as_btree = |side| {
            BTreeMap::from_iter(
                set.iter_by_vertical_edge(side)
                    .map(|(edge, mosaics)| (*edge, BTreeSet::from_iter(mosaics))),
            )
        };
        let iter_by_horizontal_edge_as_btree = |side| {
            BTreeMap::from_iter(
                set.iter_by_horizontal_edge(side)
                    .map(|(edge, mosaics)| (*edge, BTreeSet::from_iter(mosaics))),
            )
        };
        assert_eq!(
            iter_by_vertical_edge_as_btree(VerticalSide::Right),
            btree_map![
                edge_aa => btree_set![
                    (0, half_turn),
                ],
                edge_bv => btree_set![
                    (0, identity),
                ],
            ],
        );
        assert_eq!(
            iter_by_horizontal_edge_as_btree(HorizontalSide::Top),
            btree_map![
                edge_aaa => btree_set![
                    (0, identity),
                ],
                edge_sor => btree_set![
                    (0, half_turn),
                ],
            ],
        );
        assert_eq!(
            iter_by_vertical_edge_as_btree(VerticalSide::Left),
            btree_map![
                edge_aa => btree_set![
                    (0, identity),
                ],
                edge_bv => btree_set![
                    (0, half_turn),
                ],
            ],
        );
        assert_eq!(
            iter_by_horizontal_edge_as_btree(HorizontalSide::Bottom),
            btree_map![
                edge_aaa => btree_set![
                    (0, half_turn),
                ],
                edge_sor => btree_set![
                    (0, identity),
                ],
            ],
        );

        assert_eq!(
            BTreeSet::from_iter(set.query_vertical(VerticalSide::Right, &edge_bv)),
            btree_set![(0, identity)],
        );
        assert_eq!(
            BTreeSet::from_iter(set.query_vertical(VerticalSide::Right, &edge_aa)),
            btree_set![(0, half_turn)],
        );
        assert_eq!(
            BTreeSet::from_iter(set.query_horizontal(HorizontalSide::Top, &edge_aaa)),
            btree_set![(0, identity)],
        );
        assert_eq!(
            BTreeSet::from_iter(set.query_horizontal(HorizontalSide::Top, &edge_sor)),
            btree_set![(0, half_turn)],
        );
        assert_eq!(
            BTreeSet::from_iter(set.query_vertical(VerticalSide::Left, &edge_bv)),
            btree_set![(0, half_turn)],
        );
        assert_eq!(
            BTreeSet::from_iter(set.query_vertical(VerticalSide::Left, &edge_aa)),
            btree_set![(0, identity)],
        );
        assert_eq!(
            BTreeSet::from_iter(set.query_horizontal(HorizontalSide::Bottom, &edge_aaa)),
            btree_set![(0, half_turn)],
        );
        assert_eq!(
            BTreeSet::from_iter(set.query_horizontal(HorizontalSide::Bottom, &edge_sor)),
            btree_set![(0, identity)],
        );
    }
}
