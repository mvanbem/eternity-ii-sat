use bitint::prelude::*;
use std::io::{self, Write};

use crate::{Color, RotatedTile, Rotation, Tile};

#[derive(Clone, Copy)]
pub struct Literal(isize);

impl Literal {
    pub fn positive<V>(variable: V) -> Self
    where
        V: Into<usize>,
    {
        Self(variable.into() as isize)
    }

    pub fn negative<V>(variable: V) -> Self
    where
        V: Into<usize>,
    {
        Self(-(variable.into() as isize))
    }
}

pub struct BinaryClause {
    literals: [Literal; 2],
}

impl BinaryClause {
    pub fn new(a: Literal, b: Literal) -> Self {
        Self { literals: [a, b] }
    }

    pub fn print_dimacs_fragment<W: Write>(&self, mut w: W) -> io::Result<()> {
        writeln!(w, "{} {} 0", self.literals[0].0, self.literals[1].0)
    }
}

pub struct LongClause {
    literals: Vec<Literal>,
}

impl LongClause {
    pub fn new(literals: Vec<Literal>) -> Self {
        Self { literals }
    }

    pub fn print_dimacs_fragment<W: Write>(&self, mut w: W) -> io::Result<()> {
        for (i, literal) in self.literals.iter().copied().enumerate() {
            if i > 0 {
                write!(w, " ")?;
            }
            write!(w, "{}", literal.0)?;
        }
        writeln!(w, " 0")
    }
}

#[derive(Default)]
pub struct Clauses {
    binary: Vec<BinaryClause>,
    long: Vec<LongClause>,
}

impl Clauses {
    pub fn push_binary(&mut self, a: Literal, b: Literal) {
        self.binary.push(BinaryClause::new(a, b));
    }

    pub fn push_long(&mut self, literals: Vec<Literal>) {
        self.long.push(LongClause::new(literals));
    }

    pub fn push_unit(&mut self, literal: Literal) {
        self.long.push(LongClause::new(vec![literal]));
    }

    pub fn len(&self) -> usize {
        self.binary.len() + self.long.len()
    }

    pub fn print_dimacs_fragment<W: Write>(&self, mut w: W) -> io::Result<()> {
        for clause in &self.binary {
            clause.print_dimacs_fragment(w.by_ref())?;
        }
        for clause in &self.long {
            clause.print_dimacs_fragment(w.by_ref())?;
        }
        Ok(())
    }

    pub fn emit_at_most_one_of<V>(&mut self, variables: &[V])
    where
        V: Copy + Into<usize>,
    {
        for (i, a) in variables[1..].iter().copied().enumerate() {
            for b in variables[..i].iter().copied() {
                self.push_binary(Literal::negative(a), Literal::negative(b));
            }
        }
    }

    pub fn emit_at_least_one_of<V>(&mut self, variables: &[V])
    where
        V: Copy + Into<usize>,
    {
        self.push_long(
            variables
                .iter()
                .copied()
                .map(|v| Literal::positive(v))
                .collect(),
        );
    }
}

/// Coordinate system:
///
/// - X denotes column, 0..16, increasing from left to right.
/// - Y denotes row, 0..16, increasing from top to bottom.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Variable(usize);

#[bitint_literals]
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

    pub fn for_tile_placement(x: U4, y: U4, rotated_tile: RotatedTile) -> Self {
        Self(
            16384 * y.to_primitive() as usize
                + 1024 * x.to_primitive() as usize
                + 4 * rotated_tile.tile.to_primitive() as usize
                + rotated_tile.rotation.to_primitive() as usize
                + Self::TILE_PLACEMENT_BASE,
        )
    }

    pub fn for_right_edge_color(x: U4, y: U4, color: Color) -> Self {
        assert!(x < 15_U4);
        assert!(color.is_valid_non_border_color());
        Self(
            330 * y.to_primitive() as usize
                + 22 * x.to_primitive() as usize
                + (color.to_primitive() as usize - 1)
                + Self::RIGHT_EDGE_COLOR_BASE,
        )
    }

    pub fn for_bottom_edge_color(x: U4, y: U4, color: Color) -> Self {
        assert!(y < 15_U4);
        assert!(color.is_valid_non_border_color());
        Self(
            352 * y.to_primitive() as usize
                + 22 * x.to_primitive() as usize
                + (color.to_primitive() as usize - 1)
                + Self::BOTTOM_EDGE_COLOR_BASE,
        )
    }

    pub fn kind(self) -> VariableKind {
        if self.0 < Self::RIGHT_EDGE_COLOR_BASE {
            let i = self.0 - Self::TILE_PLACEMENT_BASE;
            VariableKind::TilePlacement {
                x: U4::new_masked((i / 1024) as u8),
                y: U4::new_masked((i / 16384) as u8),
                rotated_tile: RotatedTile {
                    tile: Tile::from_primitive((i / 4) as u8),
                    rotation: Rotation::new_masked(i as u8),
                },
            }
        } else if self.0 < Self::BOTTOM_EDGE_COLOR_BASE {
            let i = self.0 - Self::RIGHT_EDGE_COLOR_BASE;
            VariableKind::RightEdgeColor {
                x: U4::new_masked((i / 22 % 15) as u8),
                y: U4::new_masked((i / 330) as u8),
                color: Color::new_masked((i % 22 + 1) as u8),
            }
        } else {
            let i = self.0 - Self::BOTTOM_EDGE_COLOR_BASE;
            VariableKind::BottomEdgeColor {
                x: U4::new_masked((i / 22) as u8),
                y: U4::new_masked((i / 352) as u8),
                color: Color::new_masked((i % 22 + 1) as u8),
            }
        }
    }
}

impl From<usize> for Variable {
    fn from(index: usize) -> Self {
        Self(index)
    }
}

impl From<Variable> for usize {
    fn from(variable: Variable) -> usize {
        variable.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VariableKind {
    TilePlacement {
        x: U4,
        y: U4,
        rotated_tile: RotatedTile,
    },
    RightEdgeColor {
        x: U4,
        y: U4,
        color: Color,
    },
    BottomEdgeColor {
        x: U4,
        y: U4,
        color: Color,
    },
}

#[bitint_literals]
#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use bitint::prelude::*;
    use strum::IntoEnumIterator;

    use crate::{Color, RotatedTile, Rotation, Tile};

    use super::{Variable, VariableKind};

    #[test]
    fn variable_encoding_is_unique_and_round_trips() {
        let mut variables = HashSet::new();
        for x in 0..16 {
            let x = U4::new_masked(x);
            for y in 0..16 {
                let y = U4::new_masked(y);
                for tile in Tile::values() {
                    for rotation in Rotation::iter() {
                        let rotated_tile = RotatedTile { tile, rotation };
                        let kind = VariableKind::TilePlacement { x, y, rotated_tile };
                        let variable = Variable::for_tile_placement(x, y, rotated_tile);
                        assert_eq!(variable.kind(), kind);
                        variables.insert(variable);
                    }
                }
                for color in Color::iter() {
                    if color != Color::EXTERIOR {
                        if x < 15_U4 {
                            let kind = VariableKind::RightEdgeColor { x, y, color };
                            let variable = Variable::for_right_edge_color(x, y, color);
                            assert_eq!(variable.kind(), kind);
                            variables.insert(variable);
                        }
                        if y < 15_U4 {
                            let kind = VariableKind::BottomEdgeColor { x, y, color };
                            let variable = Variable::for_bottom_edge_color(x, y, color);
                            assert_eq!(variable.kind(), kind);
                            variables.insert(variable);
                        }
                    }
                }
            }
        }
        assert_eq!(Variable::COUNT, variables.len());
    }
}
