use std::ops::{Add, Sub};

pub mod sat;

// Coordinate system:
//   X denotes column, 0..16, increasing from left to right.
//   Y denotes row, 0..16, increasing from top to bottom.

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Edge(u8);

impl Edge {
    pub const VALUES: [Edge; 4] = [Self::TOP, Self::RIGHT, Self::BOTTOM, Self::LEFT];
    pub const TOP: Edge = Edge(0);
    pub const RIGHT: Edge = Edge(1);
    pub const BOTTOM: Edge = Edge(2);
    pub const LEFT: Edge = Edge(3);

    pub fn index(self) -> usize {
        self.0 as usize
    }
}

impl Add<Rotation> for Edge {
    type Output = Self;

    fn add(self, rhs: Rotation) -> Self {
        Self(self.0.wrapping_add(rhs.0) & 3)
    }
}

impl Sub<Rotation> for Edge {
    type Output = Self;

    fn sub(self, rhs: Rotation) -> Self {
        Self(self.0.wrapping_sub(rhs.0) & 3)
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Rotation(u8);

impl Rotation {
    pub const VALUES: [Rotation; 4] = [Rotation(0), Rotation(1), Rotation(2), Rotation(3)];
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Color(u8);

impl Color {
    pub const GRAY: Color = Color(b'a');

    pub fn values() -> impl Iterator<Item = Color> {
        (b'a'..=b'w').into_iter().map(|i| Color(i))
    }

    pub fn as_u8(self) -> u8 {
        self.0
    }

    pub fn index(self) -> usize {
        (self.0 - b'a') as usize
    }

    pub fn from_index(index: usize) -> Color {
        let c = b'a' + index as u8;
        debug_assert!(c <= b'w');
        Color(c)
    }
}

// Ranges from 0..256.
// Tile 0 is comprised of edges in TILE_EDGES[0..4].
// Tile 1 is comprised of edges in TILE_EDGES[4..8].
// ...
// Tile 255 is comprised of edges in TILE_EDGES[1020..1024].
//
// The order of edges is top, right, bottom, left.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tile(u8);

impl Tile {
    const EDGES: &'static str = "ajraaftjabvfafpbajefarcjablrajubafpjanefajmnabpjafebabifafsbaabfruratiouvvsiptuvewstcugwlwouuicwpdlieiidmocipctoelecigoesccgbabcrifaootisddoudmdsggdlddgopddcluplmwlimemicswttpcehutotvhcwetbajvftratuttdchumstcgvvsdwvvdwvwuhewwluhedvlsmsdptemupwtvvdcepemjarprijatdoihltdtmolvwomvcwwvcieesscuhssvtihshltetehwdltdhcdeighrarcjhjaoelhtileoliiosclwcssitpcscptscicilmclshlehhslgvhcwvgppvwrafpjmbalhmmlluhimulcmimsggmpppgpmhpiovmmwpohsewhwqsvwiwvovwvqdofafqbgbamhtguqghucmqitecglstpeclhmsevlompgtlehvgqtohievtvpmedutpfafubhfatplhgiipmqgiemdqsdpmclidsgglodegtohdvohoocsovcucmgoctikgfafiflnalqpliluqghvlddwhpdqdikgdgepkewiehodwhccosugcuuouoegukwlefarwnqnapdhqutkdvkqtwedkqswegpgspuopihpudqchcueqgqkuousqgpmulswprarsncnahqvckwoqqwuwdmewwvdmgttvovmtpkivcslkepkskuupsqgumsgqwsdsrajsnhfavqwhogkqukvgeowkdesothqemklhimwklklmkqtkuikqgsuigtvsdvktjanvfdnawlqdkgqlvlqgwqklsuhqqvqulimvwkpiloektkuokdckuqkdvpuqkompnajongbaqhqgqmkhqkkmkwgkhoqwqviomokvpmkoeqtmumiqcdhmkqedukwqmpkkjajpbraaqnarkjankbajgnabqbaninabkbankjabtnajiranhnarenanwrankrarjaar";

    pub fn values() -> impl Iterator<Item = Tile> {
        (0..=255).into_iter().map(|i| Tile(i))
    }

    pub fn edge_color(self, edge: Edge) -> Color {
        Color(Self::EDGES.as_bytes()[4 * self.0 as usize + edge.0 as usize])
    }

    pub fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct RotatedTile {
    pub tile: Tile,
    pub rotation: Rotation,
}

impl RotatedTile {
    pub fn edge_color(self, edge: Edge) -> Color {
        self.tile.edge_color(edge - self.rotation)
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Variable(usize);

impl Variable {
    const TILE_PLACEMENT_BASE: usize = 1;
    const TILE_PLACEMENT_COUNT: usize = 16 * 16 * 1024;

    const RIGHT_EDGE_COLOR_BASE: usize = Self::TILE_PLACEMENT_BASE + Self::TILE_PLACEMENT_COUNT;
    const RIGHT_EDGE_COLOR_COUNT: usize = 15 * 16 * 22;

    const BOTTOM_EDGE_COLOR_BASE: usize =
        Self::RIGHT_EDGE_COLOR_BASE + Self::RIGHT_EDGE_COLOR_COUNT;
    const BOTTOM_EDGE_COLOR_COUNT: usize = 16 * 15 * 22;

    // NOTE: Subtract one because count is zero-based, but variable indices are one-based.
    pub const COUNT: usize = Self::BOTTOM_EDGE_COLOR_BASE + Self::BOTTOM_EDGE_COLOR_COUNT - 1;

    // x: 0..16 (16 values)
    // y: 0..16 (16 values)
    // rotated_tile (1024 values)
    pub fn for_tile_placement(x: usize, y: usize, rotated_tile: RotatedTile) -> Self {
        debug_assert!(x < 16);
        debug_assert!(y < 16);
        debug_assert!(rotated_tile.rotation.0 < 4);
        Self(
            16384 * x
                + 1024 * y
                + 4 * rotated_tile.tile.0 as usize
                + rotated_tile.rotation.0 as usize
                + Self::TILE_PLACEMENT_BASE,
        )
    }

    // x: 0..15 (15 values)
    // y: 0..16 (16 values)
    // color: Color(b'a')..Color(b'w') (23 values)
    pub fn for_right_edge_color(x: usize, y: usize, color: Color) -> Self {
        debug_assert!(x < 15);
        debug_assert!(y < 16);
        debug_assert!(color.index() > 0 && color.index() < 23);
        Self(352 * x + 22 * y + (color.index() - 1) + Self::RIGHT_EDGE_COLOR_BASE)
    }

    // x: 0..16 (16 values)
    // y: 0..15 (15 values)
    // color: Color(b'a')..Color(b'w') (23 values)
    pub fn for_bottom_edge_color(x: usize, y: usize, color: Color) -> Self {
        debug_assert!(x < 16);
        debug_assert!(y < 15);
        debug_assert!(color.index() > 0 && color.index() < 23);
        Self(330 * x + 22 * y + (color.index() - 1) + Self::BOTTOM_EDGE_COLOR_BASE)
    }

    pub fn kind(self) -> VariableKind {
        if self.0 < Self::RIGHT_EDGE_COLOR_BASE {
            let i = self.0 - Self::TILE_PLACEMENT_BASE;
            VariableKind::TilePlacement {
                x: i / 16384,
                y: i / 1024 % 16,
                rotated_tile: RotatedTile {
                    tile: Tile((i / 4 % 256) as u8),
                    rotation: Rotation((i % 4) as u8),
                },
            }
        } else if self.0 < Self::BOTTOM_EDGE_COLOR_BASE {
            let i = self.0 - Self::RIGHT_EDGE_COLOR_BASE;
            VariableKind::RightEdgeColor {
                x: i / 352,
                y: i / 22 % 16,
                color: Color::from_index(i % 22 + 1),
            }
        } else {
            let i = self.0 - Self::BOTTOM_EDGE_COLOR_BASE;
            VariableKind::BottomEdgeColor {
                x: i / 330,
                y: i / 22 % 15,
                color: Color::from_index(i % 22 + 1),
            }
        }
    }
}

impl From<usize> for Variable {
    fn from(index: usize) -> Self {
        Self(index)
    }
}

impl Into<usize> for Variable {
    fn into(self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy, Debug)]
pub enum VariableKind {
    TilePlacement {
        x: usize,
        y: usize,
        rotated_tile: RotatedTile,
    },
    RightEdgeColor {
        x: usize,
        y: usize,
        color: Color,
    },
    BottomEdgeColor {
        x: usize,
        y: usize,
        color: Color,
    },
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::{Color, Edge, RotatedTile, Rotation, Tile, Variable};

    #[test]
    fn variable_encoding_is_unique() {
        let mut variables = HashSet::new();
        for x in 0..16 {
            for y in 0..16 {
                for tile in Tile::values() {
                    for rotation in Rotation::VALUES {
                        variables.insert(Variable::for_tile_placement(
                            x,
                            y,
                            RotatedTile { rotation, tile },
                        ));
                    }
                }
                for color in Color::values() {
                    if color != Color::GRAY {
                        if x < 15 {
                            variables.insert(Variable::for_right_edge_color(x, y, color));
                        }
                        if y < 15 {
                            variables.insert(Variable::for_bottom_edge_color(x, y, color));
                        }
                    }
                }
            }
        }
        assert_eq!(Variable::COUNT, variables.len());
    }

    #[test]
    fn edge_colors() {
        assert_eq!(Color(b'a'), Tile(0).edge_color(Edge(0)));
        assert_eq!(Color(b'j'), Tile(0).edge_color(Edge(1)));
        assert_eq!(Color(b'r'), Tile(0).edge_color(Edge(2)));
        assert_eq!(Color(b'a'), Tile(0).edge_color(Edge(3)));

        assert_eq!(Color(b'b'), Tile(128).edge_color(Edge(0)));
        assert_eq!(Color(b'h'), Tile(128).edge_color(Edge(1)));
        assert_eq!(Color(b'f'), Tile(128).edge_color(Edge(2)));
        assert_eq!(Color(b'a'), Tile(128).edge_color(Edge(3)));
    }
}
