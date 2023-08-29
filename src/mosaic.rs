use std::cmp::Ordering;
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::Hash;
use std::iter::repeat;
use std::ops::{Add, AddAssign};

use crate::edge::ArrayEdge;
use crate::rectangular::{
    HorizontalSide, RectangularRotation, RectangularSide, SideExt, VerticalSide,
};
use crate::{RotatedTile, Rotation, Side, Tile};

pub trait MosaicBounds: Clone + Debug + PartialEq + Eq + PartialOrd + Ord + Hash {}

impl<T: Clone + Debug + PartialEq + Eq + PartialOrd + Ord + Hash> MosaicBounds for T {}

pub trait RectangularMosaic<const W: usize, const H: usize>: MosaicBounds {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn get(&self, x: usize, y: usize) -> RotatedTile;

    fn vertical_edge(&self, side: VerticalSide) -> ArrayEdge<H> {
        let mut edge = ArrayEdge::default();
        match side {
            VerticalSide::Right => {
                for y in 0..H {
                    edge[y] = self.get(W - 1, y).color(Side::Right);
                }
            }
            VerticalSide::Left => {
                for y in 0..H {
                    edge[y] = self.get(0, H - 1 - y).color(Side::Left);
                }
            }
        }
        edge
    }

    fn horizontal_edge(&self, side: HorizontalSide) -> ArrayEdge<W> {
        let mut edge = ArrayEdge::default();
        match side {
            HorizontalSide::Bottom => {
                for x in 0..W {
                    edge[x] = self.get(W - 1 - x, H - 1).color(Side::Bottom);
                }
            }
            HorizontalSide::Top => {
                for x in 0..W {
                    edge[x] = self.get(x, 0).color(Side::Top);
                }
            }
        }
        edge
    }

    type WithRectangularRotation<'a>: RectangularMosaic<W, H> + 'a
    where
        Self: 'a;

    fn with_rectangular_rotation(
        &self,
        rotation: RectangularRotation,
    ) -> Self::WithRectangularRotation<'_>;

    fn display(&self, indent: usize) -> MosaicDisplay<W, H, Self> {
        MosaicDisplay {
            mosaic: self,
            indent,
        }
    }

    fn to_array_mosaic(&self) -> ArrayMosaic<W, H> {
        let mut result = ArrayMosaic {
            tiles: [[RotatedTile::ZERO; W]; H],
        };
        for y in 0..H {
            for x in 0..W {
                result.tiles[y][x] = self.get(x, y);
            }
        }
        result
    }
}

pub trait SquareMosaic<const N: usize>: RectangularMosaic<N, N> {
    fn edge(&self, side: Side) -> ArrayEdge<N> {
        match side.to_rectangular() {
            RectangularSide::Vertical(side) => self.vertical_edge(side),
            RectangularSide::Horizontal(side) => self.horizontal_edge(side),
        }
    }

    type WithSquareRotation<'a>: SquareMosaic<N> + 'a
    where
        Self: 'a;

    fn with_square_rotation(&self, rotation: Rotation) -> Self::WithSquareRotation<'_>;
}

pub struct MosaicDisplay<'a, const W: usize, const H: usize, M: RectangularMosaic<W, H>> {
    mosaic: &'a M,
    indent: usize,
}

impl<'a, const W: usize, const H: usize, M: RectangularMosaic<W, H>> Display
    for MosaicDisplay<'a, W, H, M>
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let indent = String::from_iter(repeat(' ').take(self.indent));
        for y in 0..H {
            let write_top_row = |f: &mut Formatter| {
                write!(f, "{indent}")?;
                for x in 0..W {
                    let rotated_tile = self.mosaic.get(x, y);
                    let left_is_border = rotated_tile.color(Side::Left).is_border();
                    let top_is_border = rotated_tile.color(Side::Top).is_border();
                    let right_is_border = rotated_tile.color(Side::Right).is_border();
                    let top_left = match (left_is_border, top_is_border) {
                        (true, _) => "▄",
                        (false, true) => "▗",
                        (false, false) => "┌",
                    };
                    let top = match top_is_border {
                        true => "▄▄▄",
                        false => "───",
                    };
                    let top_right = match (top_is_border, right_is_border) {
                        (_, true) => "▄",
                        (true, false) => "▖",
                        (false, false) => "┐",
                    };
                    write!(
                        f,
                        "{top_left}{top}{:^2}{top}{top_right}",
                        rotated_tile.color(Side::Top)
                    )?;
                }
                writeln!(f)?;
                write!(f, "{indent}")?;
                Ok(())
            };
            let write_fill_row =
                |f: &mut Formatter, arrow_row: Option<u8>, clue_row: Option<u8>| {
                    for x in 0..W {
                        let rotated_tile = self.mosaic.get(x, y);
                        let left_is_border = rotated_tile.color(Side::Left).is_border();
                        let right_is_border = rotated_tile.color(Side::Right).is_border();
                        let left = match left_is_border {
                            true => "█",
                            false => "│",
                        };
                        let left_arrow = match (arrow_row, rotated_tile.rotation) {
                            (Some(0), Rotation::Identity) => "▴",
                            (Some(_), Rotation::QuarterTurnLeft) => "◂",
                            (Some(1), Rotation::HalfTurn) => "▾",
                            _ => " ",
                        };
                        let clue = match (clue_row, rotated_tile.tile.to_primitive()) {
                            (Some(0), 76 | 125 | 135 | 179 | 211) => "CLUE",
                            (Some(1), 76) => " C3 ",
                            (Some(1), 179) => "C14 ",
                            (Some(1), 135) => " I8 ",
                            (Some(1), 211) => " N3 ",
                            (Some(1), 125) => "N14 ",
                            _ => "    ",
                        };
                        let right_arrow = match (arrow_row, rotated_tile.rotation) {
                            (Some(0), Rotation::Identity) => "▴",
                            (Some(_), Rotation::QuarterTurnRight) => "▸",
                            (Some(1), Rotation::HalfTurn) => "▾",
                            _ => " ",
                        };
                        let right: &str = match right_is_border {
                            true => "█",
                            false => "│",
                        };
                        write!(f, "{left} {left_arrow}{clue}{right_arrow} {right}")?;
                    }
                    writeln!(f)?;
                    write!(f, "{indent}")?;
                    Ok(())
                };
            let write_middle_row = |f: &mut Formatter| {
                for x in 0..W {
                    let rotated_tile = self.mosaic.get(x, y);
                    write!(
                        f,
                        "{}{:^8}{}",
                        rotated_tile.color(Side::Left),
                        rotated_tile.tile.to_primitive(),
                        rotated_tile.color(Side::Right),
                    )?;
                }
                writeln!(f)?;
                write!(f, "{indent}")?;
                Ok(())
            };
            let write_bottom_row = |f: &mut Formatter| {
                for x in 0..W {
                    let rotated_tile = self.mosaic.get(x, y);
                    let left_is_border = rotated_tile.color(Side::Left).is_border();
                    let bottom_is_border = rotated_tile.color(Side::Bottom).is_border();
                    let right_is_border = rotated_tile.color(Side::Right).is_border();
                    let bottom_left = match (left_is_border, bottom_is_border) {
                        (true, _) => "▀",
                        (false, true) => "▝",
                        (false, false) => "└",
                    };
                    let bottom = match bottom_is_border {
                        true => "▀▀▀",
                        false => "───",
                    };
                    let bottom_right = match (bottom_is_border, right_is_border) {
                        (_, true) => "▀",
                        (true, false) => "▘",
                        (false, false) => "┘",
                    };
                    write!(
                        f,
                        "{bottom_left}{bottom}{:^2}{bottom}{bottom_right}",
                        rotated_tile.color(Side::Bottom),
                    )?;
                }
                writeln!(f)?;
                Ok(())
            };

            write_top_row(f)?;
            write_fill_row(f, Some(0), Some(0))?;
            write_middle_row(f)?;
            write_fill_row(f, Some(1), Some(1))?;
            write_bottom_row(f)?;
        }
        Ok(())
    }
}

#[derive(Debug, Hash)]
pub struct RotatedRectangularMosaic<'a, const W: usize, const H: usize, M: RectangularMosaic<W, H>>
{
    pub mosaic: &'a M,
    pub rotation: RectangularRotation,
}

impl<'a, const W: usize, const H: usize, M: RectangularMosaic<W, H>> RectangularMosaic<W, H>
    for RotatedRectangularMosaic<'a, W, H, M>
{
    fn width(&self) -> usize {
        W
    }

    fn height(&self) -> usize {
        H
    }

    fn get(&self, x: usize, y: usize) -> RotatedTile {
        (match self.rotation {
            RectangularRotation::Identity => self.mosaic.get(x, y),
            RectangularRotation::HalfTurn => self.mosaic.get(W - 1 - x, H - 1 - y),
        }) + self.rotation.to_square()
    }

    type WithRectangularRotation<'b> = RotatedRectangularMosaic<'b, W, H, M>
    where
        Self: 'b;

    fn with_rectangular_rotation(
        &self,
        rotation: RectangularRotation,
    ) -> Self::WithRectangularRotation<'_> {
        RotatedRectangularMosaic {
            mosaic: self.mosaic,
            rotation: self.rotation + rotation,
        }
    }
}

impl<'a, const W: usize, const H: usize, M: RectangularMosaic<W, H>> Clone
    for RotatedRectangularMosaic<'a, W, H, M>
{
    fn clone(&self) -> Self {
        Self {
            mosaic: self.mosaic,
            rotation: self.rotation,
        }
    }
}

impl<'a, const W: usize, const H: usize, M: RectangularMosaic<W, H>> Copy
    for RotatedRectangularMosaic<'a, W, H, M>
{
}

impl<'a, const W: usize, const H: usize, M: RectangularMosaic<W, H>> PartialEq
    for RotatedRectangularMosaic<'a, W, H, M>
{
    fn eq(&self, other: &Self) -> bool {
        for y in 0..H {
            for x in 0..W {
                if self.get(x, y) != other.get(x, y) {
                    return false;
                }
            }
        }
        true
    }
}

impl<'a, const W: usize, const H: usize, M: RectangularMosaic<W, H>> Eq
    for RotatedRectangularMosaic<'a, W, H, M>
{
}

impl<'a, const W: usize, const H: usize, M: RectangularMosaic<W, H>> PartialOrd
    for RotatedRectangularMosaic<'a, W, H, M>
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a, const W: usize, const H: usize, M: RectangularMosaic<W, H>> Ord
    for RotatedRectangularMosaic<'a, W, H, M>
{
    fn cmp(&self, other: &Self) -> Ordering {
        for y in 0..H {
            for x in 0..W {
                match self.get(x, y).cmp(&other.get(x, y)) {
                    Ordering::Equal => (),
                    x => return x,
                }
            }
        }
        Ordering::Equal
    }
}

impl<'a, const W: usize, const H: usize, M: RectangularMosaic<W, H>> From<&'a M>
    for RotatedRectangularMosaic<'a, W, H, M>
{
    fn from(value: &'a M) -> Self {
        Self {
            mosaic: value,
            rotation: RectangularRotation::Identity,
        }
    }
}

impl<'a, const W: usize, const H: usize, M: RectangularMosaic<W, H>> Add<RectangularRotation>
    for RotatedRectangularMosaic<'a, W, H, M>
{
    type Output = Self;

    fn add(mut self, rhs: RectangularRotation) -> Self {
        self += rhs;
        self
    }
}

impl<'a, const W: usize, const H: usize, M: RectangularMosaic<W, H>> AddAssign<RectangularRotation>
    for RotatedRectangularMosaic<'a, W, H, M>
{
    fn add_assign(&mut self, rhs: RectangularRotation) {
        self.rotation += rhs;
    }
}

#[derive(Debug, Hash)]
pub struct RotatedSquareMosaic<'a, const N: usize, M: SquareMosaic<N>> {
    pub mosaic: &'a M,
    pub rotation: Rotation,
}

impl<'a, const N: usize, M: SquareMosaic<N>> RectangularMosaic<N, N>
    for RotatedSquareMosaic<'a, N, M>
{
    fn width(&self) -> usize {
        N
    }

    fn height(&self) -> usize {
        N
    }

    fn get(&self, x: usize, y: usize) -> RotatedTile {
        (match self.rotation {
            Rotation::Identity => self.mosaic.get(x, y),
            Rotation::QuarterTurnLeft => self.mosaic.get(N - 1 - y, x),
            Rotation::HalfTurn => self.mosaic.get(N - 1 - x, N - 1 - y),
            Rotation::QuarterTurnRight => self.mosaic.get(y, N - 1 - x),
        }) + self.rotation
    }

    type WithRectangularRotation<'b> = RotatedSquareMosaic<'b, N, M>
    where
        Self: 'b;

    fn with_rectangular_rotation(
        &self,
        rotation: RectangularRotation,
    ) -> Self::WithRectangularRotation<'_> {
        RotatedSquareMosaic {
            mosaic: self.mosaic,
            rotation: self.rotation + rotation.to_square(),
        }
    }
}

impl<'a, const N: usize, M: SquareMosaic<N>> SquareMosaic<N> for RotatedSquareMosaic<'a, N, M> {
    type WithSquareRotation<'b> = RotatedSquareMosaic<'b, N, M>
    where
        Self: 'b;

    fn with_square_rotation(&self, rotation: Rotation) -> Self::WithSquareRotation<'_> {
        RotatedSquareMosaic {
            mosaic: self.mosaic,
            rotation: self.rotation + rotation,
        }
    }
}

impl<'a, const N: usize, M: SquareMosaic<N>> Clone for RotatedSquareMosaic<'a, N, M> {
    fn clone(&self) -> Self {
        Self {
            mosaic: self.mosaic,
            rotation: self.rotation,
        }
    }
}

impl<'a, const N: usize, M: SquareMosaic<N>> Copy for RotatedSquareMosaic<'a, N, M> {}

impl<'a, const N: usize, M: SquareMosaic<N>> PartialEq for RotatedSquareMosaic<'a, N, M> {
    fn eq(&self, other: &Self) -> bool {
        for y in 0..N {
            for x in 0..N {
                if self.get(x, y) != other.get(x, y) {
                    return false;
                }
            }
        }
        true
    }
}

impl<'a, const N: usize, M: SquareMosaic<N>> Eq for RotatedSquareMosaic<'a, N, M> {}

impl<'a, const N: usize, M: SquareMosaic<N>> PartialOrd for RotatedSquareMosaic<'a, N, M> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a, const N: usize, M: SquareMosaic<N>> Ord for RotatedSquareMosaic<'a, N, M> {
    fn cmp(&self, other: &Self) -> Ordering {
        for y in 0..N {
            for x in 0..N {
                match self.get(x, y).cmp(&other.get(x, y)) {
                    Ordering::Equal => (),
                    x => return x,
                }
            }
        }
        Ordering::Equal
    }
}

impl<'a, const N: usize, M: SquareMosaic<N>> From<&'a M> for RotatedSquareMosaic<'a, N, M> {
    fn from(value: &'a M) -> Self {
        Self {
            mosaic: value,
            rotation: Rotation::Identity,
        }
    }
}

impl<'a, const N: usize, M: SquareMosaic<N>> Add<Rotation> for RotatedSquareMosaic<'a, N, M> {
    type Output = Self;

    fn add(mut self, rhs: Rotation) -> Self {
        self += rhs;
        self
    }
}

impl<'a, const N: usize, M: SquareMosaic<N>> AddAssign<Rotation> for RotatedSquareMosaic<'a, N, M> {
    fn add_assign(&mut self, rhs: Rotation) {
        self.rotation += rhs;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArrayMosaic<const W: usize, const H: usize> {
    pub tiles: [[RotatedTile; W]; H],
}

impl<const W: usize, const H: usize> RectangularMosaic<W, H> for ArrayMosaic<W, H> {
    fn width(&self) -> usize {
        W
    }

    fn height(&self) -> usize {
        H
    }

    fn get(&self, x: usize, y: usize) -> RotatedTile {
        self.tiles[y][x]
    }

    type WithRectangularRotation<'a> = RotatedRectangularMosaic<'a, W, H, Self>
    where
        Self: 'a;

    fn with_rectangular_rotation(
        &self,
        rotation: RectangularRotation,
    ) -> Self::WithRectangularRotation<'_> {
        RotatedRectangularMosaic {
            mosaic: self,
            rotation,
        }
    }
}

impl<const N: usize> SquareMosaic<N> for ArrayMosaic<N, N> {
    type WithSquareRotation<'a> = RotatedSquareMosaic<'a, N, Self>
    where
        Self: 'a;

    fn with_square_rotation(&self, rotation: Rotation) -> Self::WithSquareRotation<'_> {
        RotatedSquareMosaic {
            mosaic: self,
            rotation,
        }
    }
}

impl<'a, const W: usize, const H: usize> Add<RectangularRotation> for &'a ArrayMosaic<W, H> {
    type Output = RotatedRectangularMosaic<'a, W, H, ArrayMosaic<W, H>>;

    fn add(self, rhs: RectangularRotation) -> Self::Output {
        RotatedRectangularMosaic {
            mosaic: self,
            rotation: rhs,
        }
    }
}

impl<'a, const N: usize> Add<Rotation> for &'a ArrayMosaic<N, N> {
    type Output = RotatedSquareMosaic<'a, N, ArrayMosaic<N, N>>;

    fn add(self, rhs: Rotation) -> Self::Output {
        RotatedSquareMosaic {
            mosaic: self,
            rotation: rhs,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PackedQuadRotatedTile {
    pub tiles: [Tile; 4],
    pub rotations: [Rotation; 4],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PackedArrayMosaic<const W: usize, const H: usize, const QW: usize, const QH: usize> {
    pub tiles: [[PackedQuadRotatedTile; QW]; QH],
}

impl<const W: usize, const H: usize, const QW: usize, const QH: usize> RectangularMosaic<W, H>
    for PackedArrayMosaic<W, H, QW, QH>
{
    fn width(&self) -> usize {
        W
    }

    fn height(&self) -> usize {
        H
    }

    fn get(&self, x: usize, y: usize) -> RotatedTile {
        let quad = &self.tiles[y / 2][x / 2];
        let index = 2 * (y % 2) + x % 2;
        RotatedTile {
            tile: quad.tiles[index],
            rotation: quad.rotations[index],
        }
    }

    type WithRectangularRotation<'a> = RotatedRectangularMosaic<'a, W, H, Self>
    where
        Self: 'a;

    fn with_rectangular_rotation(
        &self,
        rotation: RectangularRotation,
    ) -> Self::WithRectangularRotation<'_> {
        RotatedRectangularMosaic {
            mosaic: self,
            rotation,
        }
    }
}

impl<const N: usize, const QN: usize> SquareMosaic<N> for PackedArrayMosaic<N, N, QN, QN> {
    type WithSquareRotation<'a> = RotatedSquareMosaic<'a, N, Self>
    where
        Self: 'a;

    fn with_square_rotation(&self, rotation: Rotation) -> Self::WithSquareRotation<'_> {
        RotatedSquareMosaic {
            mosaic: self,
            rotation,
        }
    }
}

impl<'a, const W: usize, const H: usize, const QW: usize, const QH: usize> Add<RectangularRotation>
    for &'a PackedArrayMosaic<W, H, QW, QH>
{
    type Output = RotatedRectangularMosaic<'a, W, H, PackedArrayMosaic<W, H, QW, QH>>;

    fn add(self, rhs: RectangularRotation) -> Self::Output {
        RotatedRectangularMosaic {
            mosaic: self,
            rotation: rhs,
        }
    }
}

impl<'a, const N: usize, const QN: usize> Add<Rotation> for &'a PackedArrayMosaic<N, N, QN, QN> {
    type Output = RotatedSquareMosaic<'a, N, PackedArrayMosaic<N, N, QN, QN>>;

    fn add(self, rhs: Rotation) -> Self::Output {
        RotatedSquareMosaic {
            mosaic: self,
            rotation: rhs,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::mosaic::{RotatedRectangularMosaic, SquareMosaic};
    use crate::rectangular::{HorizontalSide, RectangularRotation, VerticalSide};
    use crate::{RotatedTile, Rotation, Side, Tile};

    use super::{ArrayMosaic, RectangularMosaic, RotatedSquareMosaic};

    #[test]
    fn to_array_mosaic() {
        let mosaic = mosaic![[0, 1, 2], [10, 11, 12]];
        assert_eq!(mosaic.to_array_mosaic(), mosaic);
    }

    #[test]
    fn rotated_square_mosaic() {
        let tile_a = Tile::from_primitive(0);
        let tile_b = Tile::from_primitive(1);
        let tile_c = Tile::from_primitive(16);
        let tile_d = Tile::from_primitive(17);
        let mosaic = ArrayMosaic {
            tiles: [
                [
                    RotatedTile {
                        tile: tile_a,
                        rotation: Rotation::Identity,
                    },
                    RotatedTile {
                        tile: tile_b,
                        rotation: Rotation::Identity,
                    },
                ],
                [
                    RotatedTile {
                        tile: tile_c,
                        rotation: Rotation::Identity,
                    },
                    RotatedTile {
                        tile: tile_d,
                        rotation: Rotation::Identity,
                    },
                ],
            ],
        };

        let rotated = RotatedSquareMosaic {
            mosaic: &mosaic,
            rotation: Rotation::Identity,
        };
        assert_eq!(rotated.get(0, 0).tile, tile_a);
        assert_eq!(rotated.get(1, 0).tile, tile_b);
        assert_eq!(rotated.get(0, 1).tile, tile_c);
        assert_eq!(rotated.get(1, 1).tile, tile_d);
        assert_eq!(rotated.get(0, 0).rotation, Rotation::Identity);
        assert_eq!(rotated.get(1, 0).rotation, Rotation::Identity);
        assert_eq!(rotated.get(0, 1).rotation, Rotation::Identity);
        assert_eq!(rotated.get(1, 1).rotation, Rotation::Identity);
        assert_eq!(rotated.edge(Side::Right).to_string(), "fi");
        assert_eq!(rotated.edge(Side::Top).to_string(), "aa");
        assert_eq!(rotated.edge(Side::Left).to_string(), "aa");
        assert_eq!(rotated.edge(Side::Bottom).to_string(), "or");

        let rotated = RotatedSquareMosaic {
            mosaic: &mosaic,
            rotation: Rotation::QuarterTurnLeft,
        };
        assert_eq!(rotated.get(0, 0).tile, tile_b);
        assert_eq!(rotated.get(1, 0).tile, tile_d);
        assert_eq!(rotated.get(0, 1).tile, tile_a);
        assert_eq!(rotated.get(1, 1).tile, tile_c);
        assert_eq!(rotated.get(0, 0).rotation, Rotation::QuarterTurnLeft);
        assert_eq!(rotated.get(1, 0).rotation, Rotation::QuarterTurnLeft);
        assert_eq!(rotated.get(0, 1).rotation, Rotation::QuarterTurnLeft);
        assert_eq!(rotated.get(1, 1).rotation, Rotation::QuarterTurnLeft);
        assert_eq!(rotated.edge(Side::Right).to_string(), "or");
        assert_eq!(rotated.edge(Side::Top).to_string(), "fi");
        assert_eq!(rotated.edge(Side::Left).to_string(), "aa");
        assert_eq!(rotated.edge(Side::Bottom).to_string(), "aa");

        let rotated = RotatedSquareMosaic {
            mosaic: &mosaic,
            rotation: Rotation::HalfTurn,
        };
        assert_eq!(rotated.get(0, 0).tile, tile_d);
        assert_eq!(rotated.get(1, 0).tile, tile_c);
        assert_eq!(rotated.get(0, 1).tile, tile_b);
        assert_eq!(rotated.get(1, 1).tile, tile_a);
        assert_eq!(rotated.get(0, 0).rotation, Rotation::HalfTurn);
        assert_eq!(rotated.get(1, 0).rotation, Rotation::HalfTurn);
        assert_eq!(rotated.get(0, 1).rotation, Rotation::HalfTurn);
        assert_eq!(rotated.get(1, 1).rotation, Rotation::HalfTurn);
        assert_eq!(rotated.edge(Side::Right).to_string(), "aa");
        assert_eq!(rotated.edge(Side::Top).to_string(), "or");
        assert_eq!(rotated.edge(Side::Left).to_string(), "fi");
        assert_eq!(rotated.edge(Side::Bottom).to_string(), "aa");

        let rotated = RotatedSquareMosaic {
            mosaic: &mosaic,
            rotation: Rotation::QuarterTurnRight,
        };
        assert_eq!(rotated.get(0, 0).tile, tile_c);
        assert_eq!(rotated.get(1, 0).tile, tile_a);
        assert_eq!(rotated.get(0, 1).tile, tile_d);
        assert_eq!(rotated.get(1, 1).tile, tile_b);
        assert_eq!(rotated.get(0, 0).rotation, Rotation::QuarterTurnRight);
        assert_eq!(rotated.get(1, 0).rotation, Rotation::QuarterTurnRight);
        assert_eq!(rotated.get(0, 1).rotation, Rotation::QuarterTurnRight);
        assert_eq!(rotated.get(1, 1).rotation, Rotation::QuarterTurnRight);
        assert_eq!(rotated.edge(Side::Right).to_string(), "aa");
        assert_eq!(rotated.edge(Side::Top).to_string(), "aa");
        assert_eq!(rotated.edge(Side::Left).to_string(), "or");
        assert_eq!(rotated.edge(Side::Bottom).to_string(), "fi");
    }

    fn right<const W: usize, const H: usize>(mosaic: impl RectangularMosaic<W, H>) -> String {
        mosaic.vertical_edge(VerticalSide::Right).to_string()
    }

    fn top<const W: usize, const H: usize>(mosaic: impl RectangularMosaic<W, H>) -> String {
        mosaic.horizontal_edge(HorizontalSide::Top).to_string()
    }

    fn left<const W: usize, const H: usize>(mosaic: impl RectangularMosaic<W, H>) -> String {
        mosaic.vertical_edge(VerticalSide::Left).to_string()
    }

    fn bottom<const W: usize, const H: usize>(mosaic: impl RectangularMosaic<W, H>) -> String {
        mosaic.horizontal_edge(HorizontalSide::Bottom).to_string()
    }

    #[test]
    fn rotated_rectangular_mosaic() {
        let tile_a = Tile::from_primitive(0);
        let tile_b = Tile::from_primitive(1);
        let tile_c = Tile::from_primitive(2);
        let tile_d = Tile::from_primitive(3);
        let tile_e = Tile::from_primitive(16);
        let tile_f = Tile::from_primitive(17);
        let tile_g = Tile::from_primitive(18);
        let tile_h = Tile::from_primitive(19);
        let tile_i = Tile::from_primitive(32);
        let tile_j = Tile::from_primitive(33);
        let tile_k = Tile::from_primitive(34);
        let tile_l = Tile::from_primitive(35);
        let mosaic = ArrayMosaic::<4, 3> {
            tiles: [
                [tile_a, tile_b, tile_c, tile_d],
                [tile_e, tile_f, tile_g, tile_h],
                [tile_i, tile_j, tile_k, tile_l],
            ]
            .into_iter()
            .map(|row| {
                row.into_iter()
                    .map(|tile| RotatedTile {
                        tile,
                        rotation: Rotation::Identity,
                    })
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap()
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap(),
        };

        let rotated = RotatedRectangularMosaic {
            mosaic: &mosaic,
            rotation: RectangularRotation::Identity,
        };
        assert_eq!(rotated.get(0, 0).tile, tile_a);
        assert_eq!(rotated.get(1, 0).tile, tile_b);
        assert_eq!(rotated.get(2, 0).tile, tile_c);
        assert_eq!(rotated.get(3, 0).tile, tile_d);
        assert_eq!(rotated.get(0, 1).tile, tile_e);
        assert_eq!(rotated.get(1, 1).tile, tile_f);
        assert_eq!(rotated.get(2, 1).tile, tile_g);
        assert_eq!(rotated.get(3, 1).tile, tile_h);
        assert_eq!(rotated.get(0, 2).tile, tile_i);
        assert_eq!(rotated.get(1, 2).tile, tile_j);
        assert_eq!(rotated.get(2, 2).tile, tile_k);
        assert_eq!(rotated.get(3, 2).tile, tile_l);
        assert_eq!(rotated.get(0, 0).rotation, Rotation::Identity);
        assert_eq!(rotated.get(1, 0).rotation, Rotation::Identity);
        assert_eq!(rotated.get(2, 0).rotation, Rotation::Identity);
        assert_eq!(rotated.get(3, 0).rotation, Rotation::Identity);
        assert_eq!(rotated.get(0, 1).rotation, Rotation::Identity);
        assert_eq!(rotated.get(1, 1).rotation, Rotation::Identity);
        assert_eq!(rotated.get(2, 1).rotation, Rotation::Identity);
        assert_eq!(rotated.get(3, 1).rotation, Rotation::Identity);
        assert_eq!(rotated.get(0, 2).rotation, Rotation::Identity);
        assert_eq!(rotated.get(1, 2).rotation, Rotation::Identity);
        assert_eq!(rotated.get(2, 2).rotation, Rotation::Identity);
        assert_eq!(rotated.get(3, 2).rotation, Rotation::Identity);
        assert_eq!(right(rotated), "ftd");
        assert_eq!(top(rotated), "aaaa");
        assert_eq!(left(rotated), "aaa");
        assert_eq!(bottom(rotated), "mdtf");

        let rotated = RotatedRectangularMosaic {
            mosaic: &mosaic,
            rotation: RectangularRotation::HalfTurn,
        };
        assert_eq!(rotated.get(0, 0).tile, tile_l);
        assert_eq!(rotated.get(1, 0).tile, tile_k);
        assert_eq!(rotated.get(2, 0).tile, tile_j);
        assert_eq!(rotated.get(3, 0).tile, tile_i);
        assert_eq!(rotated.get(0, 1).tile, tile_h);
        assert_eq!(rotated.get(1, 1).tile, tile_g);
        assert_eq!(rotated.get(2, 1).tile, tile_f);
        assert_eq!(rotated.get(3, 1).tile, tile_e);
        assert_eq!(rotated.get(0, 2).tile, tile_d);
        assert_eq!(rotated.get(1, 2).tile, tile_c);
        assert_eq!(rotated.get(2, 2).tile, tile_b);
        assert_eq!(rotated.get(3, 2).tile, tile_a);
        assert_eq!(rotated.get(0, 0).rotation, Rotation::HalfTurn);
        assert_eq!(rotated.get(1, 0).rotation, Rotation::HalfTurn);
        assert_eq!(rotated.get(2, 0).rotation, Rotation::HalfTurn);
        assert_eq!(rotated.get(3, 0).rotation, Rotation::HalfTurn);
        assert_eq!(rotated.get(0, 1).rotation, Rotation::HalfTurn);
        assert_eq!(rotated.get(1, 1).rotation, Rotation::HalfTurn);
        assert_eq!(rotated.get(2, 1).rotation, Rotation::HalfTurn);
        assert_eq!(rotated.get(3, 1).rotation, Rotation::HalfTurn);
        assert_eq!(rotated.get(0, 2).rotation, Rotation::HalfTurn);
        assert_eq!(rotated.get(1, 2).rotation, Rotation::HalfTurn);
        assert_eq!(rotated.get(2, 2).rotation, Rotation::HalfTurn);
        assert_eq!(rotated.get(3, 2).rotation, Rotation::HalfTurn);
        assert_eq!(right(rotated), "aaa");
        assert_eq!(top(rotated), "mdtf");
        assert_eq!(left(rotated), "ftd");
        assert_eq!(bottom(rotated), "aaaa");
    }
}
