use std::fmt::{self, Display, Formatter};
use std::ops::{Add, AddAssign};

use mvbitfield::prelude::*;
use strum::EnumIter;

#[macro_use]
mod macros;

pub mod edge;
pub mod mosaic;
pub mod rectangular;
pub mod report;
pub mod sat;
pub mod set;

bitfield! {
    #[derive(PartialOrd, Ord, EnumIter)]
    pub enum Side: 2 {
        Right,
        Top,
        Left,
        Bottom,
    }
}

impl Side {
    pub fn rotation_from_right(self) -> Rotation {
        self.to_bitint().into()
    }

    pub fn transform(self, rotation: Rotation) -> Self {
        self.to_bitint().wrapping_add(rotation.to_bitint()).into()
    }

    pub fn reverse_transform(self, rotation: Rotation) -> Self {
        self.to_bitint().wrapping_sub(rotation.to_bitint()).into()
    }
}

bitfield! {
    #[derive(PartialOrd, Ord, EnumIter)]
    pub enum Rotation: 2 {
        Identity,
        QuarterTurnLeft,
        HalfTurn,
        QuarterTurnRight,
    }
}

impl Add for Rotation {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        self.to_bitint().wrapping_add(rhs.to_bitint()).into()
    }
}

impl AddAssign for Rotation {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

bitfield! {
    #[derive(PartialOrd, Ord)]
    pub struct Color: 5 { .. }
}

#[bitint_literals]
impl Color {
    pub const EXTERIOR: Self = Self::from_bitint(0_U5);
    pub const BORDER_COLOR_MIN: Self = Self::from_bitint(1_U5);
    pub const BORDER_COLOR_MAX: Self = Self::from_bitint(5_U5);
    pub const INTERIOR_COLOR_MIN: Self = Self::from_bitint(6_U5);
    pub const INTERIOR_COLOR_MAX: Self = Self::from_bitint(22_U5);

    // These constants correspond to `motifs_order=jblackwood` on
    // https://e2.bucas.name.
    pub const A: Self = Self::from_bitint(0_U5);
    pub const B: Self = Self::from_bitint(1_U5);
    pub const C: Self = Self::from_bitint(2_U5);
    pub const D: Self = Self::from_bitint(3_U5);
    pub const E: Self = Self::from_bitint(4_U5);
    pub const F: Self = Self::from_bitint(5_U5);
    pub const G: Self = Self::from_bitint(6_U5);
    pub const H: Self = Self::from_bitint(7_U5);
    pub const I: Self = Self::from_bitint(8_U5);
    pub const J: Self = Self::from_bitint(9_U5);
    pub const K: Self = Self::from_bitint(10_U5);
    pub const L: Self = Self::from_bitint(11_U5);
    pub const M: Self = Self::from_bitint(12_U5);
    pub const N: Self = Self::from_bitint(13_U5);
    pub const O: Self = Self::from_bitint(14_U5);
    pub const P: Self = Self::from_bitint(15_U5);
    pub const Q: Self = Self::from_bitint(16_U5);
    pub const R: Self = Self::from_bitint(17_U5);
    pub const S: Self = Self::from_bitint(18_U5);
    pub const T: Self = Self::from_bitint(19_U5);
    pub const U: Self = Self::from_bitint(20_U5);
    pub const V: Self = Self::from_bitint(21_U5);
    pub const W: Self = Self::from_bitint(22_U5);

    pub fn iter() -> impl Iterator<Item = Self> {
        (0u8..=22u8)
            .into_iter()
            .map(|x| unsafe { Self::new_unchecked(x) })
    }

    pub fn is_border(self) -> bool {
        self == Self::EXTERIOR
    }

    pub fn is_valid_non_border_color(self) -> bool {
        self >= Self::BORDER_COLOR_MIN && self <= Self::INTERIOR_COLOR_MAX
    }

    pub const fn from_byte_char(b: u8) -> Option<Self> {
        if b >= b'a' && b <= b'w' {
            Some(unsafe { Self::new_unchecked(b - b'a') })
        } else {
            None
        }
    }

    pub fn from_char(c: char) -> Option<Self> {
        c.try_into().ok().and_then(Self::from_byte_char)
    }

    pub fn to_byte_char(self) -> u8 {
        b'a' + self.to_primitive()
    }

    pub fn to_char(self) -> char {
        self.to_byte_char() as char
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.pad(&self.to_char().to_string())
    }
}

#[cfg(test)]
mod color_tests {
    use super::*;

    #[test]
    fn iter_and_to_char() {
        assert_eq!(
            String::from_iter(Color::iter().map(|color| color.to_char())),
            "abcdefghijklmnopqrstuvw",
        );
    }
}

bitfield! {
    #[derive(PartialOrd, Ord)]
    pub struct Tile: 8 { .. }
}

impl Tile {
    // Tile 0 is comprised of edges in EDGES[0..4].
    // Tile 1 is comprised of edges in EDGES[4..8].
    // ...
    // Tile 255 is comprised of edges in EDGES[1020..1024].
    //
    // The order of edges is the same as Side: right, top, left, bottom.
    const EDGES: &'static str = "jaarfajtbafvfabpjaferajcbarljabufajpnafejanmbajpfabebafifabsaafburarituovvistpvuwetsucwgwluoiuwcdpiliediomiccpotlecegieocsgcabcbirafooitdsoddudmgsdgdlgdpoddlcpumllwmimeciwsttcphetutohvwcteabvjtfarutttcduhsmctvgsvwdvvwdwvhuwelwhudelvmsdstpmeputwvvcdpemeajprirajdtiolhdtmtlowvmocvwwcveisecshusstvhihstltehedwtlhddciehgarcrhjajeohlitelloiisolccwssticpcstpcscilicmsllhheshglhvwcgvppwvarpfmjabhlmmllhumilumcmigsmgppgpmpphoimvwmopshwewhsqwvwiovwvqvodafqfgbabhmgtquhgcuqmticelgtseplcmheslvmogplthegvtqhoeitvpvemudptafufhbafpthligpiqmigmeqddsmplcdigslgdogeotdhovohcooscvcugmcoitgkafiflfanqllpliquhglvddhwdpdqkidgegkpweeiohwdchocuscguuuoeougwkelafwrqnandpqhtudkkvtqewkdsqewpgsguppohiupqdhcucqeqgukuoqspgumslpwarsrcnanqhcvwkqowqwumdwevwmdtgvtvotmkpviscklpeskukpuqsugsmqgswsdarsjhnafqvhwgoqkkugvoekwedoshteqkmhlmikwklmlqkktiuqksgiutgsvvdtkajvndfanlwdqgklqlvgqqwlkusqhvquqilvmkwipolkektoudkkcqudkpvquokpmanojgnabhqgqmqhkkqmkwkkgohwqvqoiomvkmpokqemtmuqidcmhqkdekuqwpmkkajpjrbaanqrajknabkjangbabqnanibabknajkbantjarinanhranenarwnarkraajra";

    const PARSED_EDGES: &[Color] = &{
        let mut parsed_edges = [Color::EXTERIOR; 1024];
        let mut index = 0;
        while index < 1024 {
            if let Some(color) = Color::from_byte_char(Self::EDGES.as_bytes()[index]) {
                parsed_edges[index] = color;
            }
            index += 1;
        }
        parsed_edges
    };

    pub fn values() -> impl Iterator<Item = Tile> {
        (0..=255).into_iter().map(|i| Tile::from_primitive(i))
    }

    pub fn color(self, side: Side) -> Color {
        // Color::from_byte_char(
        //     Self::EDGES.as_bytes()[4 * self.to_primitive() as usize + side.to_primitive() as usize],
        // )
        // .unwrap()
        Self::PARSED_EDGES[4 * self.to_primitive() as usize + side.to_primitive() as usize]
    }
}

bitfield! {
    #[lsb_first]
    pub struct ExteriorMask: 4 {
        pub right: 1 as bool,
        pub top: 1 as bool,
        pub left: 1 as bool,
        pub bottom: 1 as bool,
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct RotatedTile {
    pub tile: Tile,
    pub rotation: Rotation,
}

impl RotatedTile {
    pub const ZERO: Self = Self {
        tile: Tile::ZERO,
        rotation: Rotation::Identity,
    };

    pub const MAX: Self = Self {
        tile: Tile::from_primitive(255),
        rotation: Rotation::QuarterTurnRight,
    };

    pub fn color(self, side: Side) -> Color {
        self.tile.color(side.reverse_transform(self.rotation))
    }

    pub fn exterior_mask(self) -> ExteriorMask {
        ExteriorMask::zero()
            .with_right(self.color(Side::Right) == Color::EXTERIOR)
            .with_top(self.color(Side::Top) == Color::EXTERIOR)
            .with_left(self.color(Side::Left) == Color::EXTERIOR)
            .with_bottom(self.color(Side::Bottom) == Color::EXTERIOR)
    }
}

impl Add<Rotation> for RotatedTile {
    type Output = Self;

    fn add(mut self, rhs: Rotation) -> Self {
        self += rhs;
        self
    }
}

impl AddAssign<Rotation> for RotatedTile {
    fn add_assign(&mut self, rhs: Rotation) {
        self.rotation += rhs;
    }
}

#[bitint_literals]
pub fn hints() -> impl Iterator<Item = (U4, U4, RotatedTile)> {
    [
        (
            7_U4,
            8_U4,
            RotatedTile {
                tile: Tile::from_primitive(135),
                rotation: Rotation::Identity,
            },
        ),
        (
            2_U4,
            2_U4,
            RotatedTile {
                tile: Tile::from_primitive(76),
                rotation: Rotation::Identity,
            },
        ),
        (
            13_U4,
            2_U4,
            RotatedTile {
                tile: Tile::from_primitive(179),
                rotation: Rotation::QuarterTurnLeft,
            },
        ),
        (
            2_U4,
            13_U4,
            RotatedTile {
                tile: Tile::from_primitive(211),
                rotation: Rotation::HalfTurn,
            },
        ),
        (
            13_U4,
            13_U4,
            RotatedTile {
                tile: Tile::from_primitive(125),
                rotation: Rotation::QuarterTurnRight,
            },
        ),
    ]
    .into_iter()
}

#[bitint_literals]
#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use super::{Color, RotatedTile, Rotation, Side, Tile};

    #[test]
    fn rotate_side() {
        for (t, pairs) in [
            (
                Rotation::Identity,
                [
                    (Side::Right, Side::Right),
                    (Side::Top, Side::Top),
                    (Side::Left, Side::Left),
                    (Side::Bottom, Side::Bottom),
                ],
            ),
            (
                Rotation::QuarterTurnLeft,
                [
                    (Side::Right, Side::Top),
                    (Side::Top, Side::Left),
                    (Side::Left, Side::Bottom),
                    (Side::Bottom, Side::Right),
                ],
            ),
            (
                Rotation::HalfTurn,
                [
                    (Side::Right, Side::Left),
                    (Side::Top, Side::Bottom),
                    (Side::Left, Side::Right),
                    (Side::Bottom, Side::Top),
                ],
            ),
            (
                Rotation::QuarterTurnRight,
                [
                    (Side::Right, Side::Bottom),
                    (Side::Top, Side::Right),
                    (Side::Left, Side::Top),
                    (Side::Bottom, Side::Left),
                ],
            ),
        ] {
            for (a, b) in pairs {
                assert_eq!(a.transform(t), b);
                assert_eq!(b.reverse_transform(t), a);
            }
        }
    }

    #[test]
    fn rotated_tile_colors() {
        let tile = Tile::from_primitive(0);
        for (rotation, colors) in [
            (Rotation::Identity, "jaar"),
            (Rotation::QuarterTurnLeft, "rjaa"),
            (Rotation::HalfTurn, "arja"),
            (Rotation::QuarterTurnRight, "aarj"),
        ] {
            let rotated_tile = RotatedTile { tile, rotation };
            for (expected, actual) in colors
                .chars()
                .zip(Side::iter().map(|side| rotated_tile.color(side)))
            {
                assert_eq!(Color::from_char(expected), Some(actual));
            }
        }
    }

    #[test]
    fn edge_colors() {
        let t0 = Tile::from_primitive(0);
        assert_eq!(Color::from_char('a'), Some(t0.color(Side::Top)));
        assert_eq!(Color::from_char('j'), Some(t0.color(Side::Right)));
        assert_eq!(Color::from_char('r'), Some(t0.color(Side::Bottom)));
        assert_eq!(Color::from_char('a'), Some(t0.color(Side::Left)));

        let t128 = Tile::from_primitive(128);
        assert_eq!(Color::from_char('b'), Some(t128.color(Side::Top)));
        assert_eq!(Color::from_char('h'), Some(t128.color(Side::Right)));
        assert_eq!(Color::from_char('f'), Some(t128.color(Side::Bottom)));
        assert_eq!(Color::from_char('a'), Some(t128.color(Side::Left)));
    }
}
