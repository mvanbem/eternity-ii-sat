use std::io::{stdout, BufWriter};

use anyhow::Result;
use bitint::prelude::*;
use eternity_ii::sat::{Clauses, Literal, Variable};
use eternity_ii::{hints, Color, RotatedTile, Rotation, Side, Tile};
use strum::IntoEnumIterator;

#[bitint_literals]
fn main() -> Result<()> {
    let mut clauses = Clauses::default();

    // Assign the tile placements known from published clues.
    for (x, y, rotated_tile) in hints() {
        clauses.push_unit(Literal::positive(Variable::for_tile_placement(
            x,
            y,
            rotated_tile,
        )));
        if x < 15_U4 {
            clauses.push_unit(Literal::positive(Variable::for_right_edge_color(
                x,
                y,
                rotated_tile.color(Side::Right),
            )));
        }
        if y < 15_U4 {
            clauses.push_unit(Literal::positive(Variable::for_bottom_edge_color(
                x,
                y,
                rotated_tile.color(Side::Bottom),
            )));
        }
    }

    // One rotated tile per cell.
    for y in 0..16 {
        for x in 0..16 {
            let mut variables = Vec::new();
            for tile in Tile::values() {
                for rotation in Rotation::iter() {
                    variables.push(Variable::for_tile_placement(
                        U4::new_masked(x),
                        U4::new_masked(y),
                        RotatedTile { tile, rotation },
                    ));
                }
            }
            clauses.emit_at_most_one_of(&variables);
            clauses.emit_at_least_one_of(&variables);
        }
    }

    // One use for each tile.
    for tile in Tile::values() {
        let mut variables = Vec::new();
        for y in 0..16 {
            for x in 0..16 {
                for rotation in Rotation::iter() {
                    variables.push(Variable::for_tile_placement(
                        U4::new_masked(x),
                        U4::new_masked(y),
                        RotatedTile { tile, rotation },
                    ));
                }
            }
        }
        clauses.emit_at_most_one_of(&variables);
        // No need to emit_at_least_one_of() here. The above constraint for one rotated tile per
        // cell already ensures by the pigeonhole principle that all tiles are placed.
    }

    // Imply right edge colors for tile placements.
    for y in 0..16 {
        let y = U4::new_masked(y);
        for x in 0..15 {
            let x = U4::new_masked(x);
            for tile in Tile::values() {
                for rotation in Rotation::iter() {
                    let rotated_tile = RotatedTile { tile, rotation };

                    // Right edge of the tile at (x, y).
                    let color = rotated_tile.color(Side::Right);
                    if color.is_valid_non_border_color() {
                        // placed(x, y, rotated_tile) -> right_edge_color(x, y, color)
                        clauses.push_binary(
                            Literal::negative(Variable::for_tile_placement(x, y, rotated_tile)),
                            Literal::positive(Variable::for_right_edge_color(x, y, color)),
                        );
                        for other_color in Color::iter() {
                            if other_color.is_valid_non_border_color() && other_color != color {
                                // placed(x, y, rotated_tile) -> -right_edge_color(x, y, other_color)
                                clauses.push_binary(
                                    Literal::negative(Variable::for_tile_placement(
                                        x,
                                        y,
                                        rotated_tile,
                                    )),
                                    Literal::negative(Variable::for_right_edge_color(
                                        x,
                                        y,
                                        other_color,
                                    )),
                                );
                            }
                        }
                    } else {
                        // Can't place a gray edge in the middle of the board.
                        // -placed(x, y, rotated_tile)
                        clauses.push_unit(Literal::negative(Variable::for_tile_placement(
                            x,
                            y,
                            rotated_tile,
                        )));
                    }

                    // Left edge of the tile at (x+1, y).
                    let color = rotated_tile.color(Side::Left);
                    if color.is_valid_non_border_color() {
                        // placed(x+1, y, rotated_tile) -> right_edge_color(x, y, color)
                        clauses.push_binary(
                            Literal::negative(Variable::for_tile_placement(
                                x + 1_U4,
                                y,
                                rotated_tile,
                            )),
                            Literal::positive(Variable::for_right_edge_color(x, y, color)),
                        );
                        for other_color in Color::iter() {
                            if other_color.is_valid_non_border_color() && other_color != color {
                                // placed(x+1, y, rotated_tile) -> -right_edge_color(x, y, other_color)
                                clauses.push_binary(
                                    Literal::negative(Variable::for_tile_placement(
                                        x + 1_U4,
                                        y,
                                        rotated_tile,
                                    )),
                                    Literal::negative(Variable::for_right_edge_color(
                                        x,
                                        y,
                                        other_color,
                                    )),
                                );
                            }
                        }
                    } else {
                        // Can't place a gray edge in the middle of the board.
                        // -placed(x+1, y, rotated_tile)
                        clauses.push_unit(Literal::negative(Variable::for_tile_placement(
                            x + 1_U4,
                            y,
                            rotated_tile,
                        )));
                    }
                }
            }
        }
    }

    // Imply bottom edge colors for tile placements.
    for y in 0..15 {
        let y = U4::new_masked(y);
        for x in 0..16 {
            let x = U4::new_masked(x);
            for tile in Tile::values() {
                for rotation in Rotation::iter() {
                    let rotated_tile = RotatedTile { tile, rotation };

                    // Bottom edge of the tile at (x, y).
                    let color = rotated_tile.color(Side::Bottom);
                    if color != Color::EXTERIOR {
                        // placed(x, y, rotated_tile) -> bottom_edge_color(x, y, color)
                        clauses.push_binary(
                            Literal::negative(Variable::for_tile_placement(x, y, rotated_tile)),
                            Literal::positive(Variable::for_bottom_edge_color(x, y, color)),
                        );
                        for other_color in Color::iter() {
                            if other_color != Color::EXTERIOR && other_color != color {
                                // placed(x, y, rotated_tile) -> -bottom_edge_color(x, y, other_color)
                                clauses.push_binary(
                                    Literal::negative(Variable::for_tile_placement(
                                        x,
                                        y,
                                        rotated_tile,
                                    )),
                                    Literal::negative(Variable::for_bottom_edge_color(
                                        x,
                                        y,
                                        other_color,
                                    )),
                                );
                            }
                        }
                    } else {
                        // Can't place a gray edge in the middle of the board.
                        // -placed(x, y, rotated_tile)
                        clauses.push_unit(Literal::negative(Variable::for_tile_placement(
                            x,
                            y,
                            rotated_tile,
                        )));
                    }

                    // Top edge of the tile at (x, y+1).
                    let color = rotated_tile.color(Side::Top);
                    if color != Color::EXTERIOR {
                        // placed(x, y+1, rotated_tile) -> bottom_edge_color(x, y, color)
                        clauses.push_binary(
                            Literal::negative(Variable::for_tile_placement(
                                x,
                                y + 1_U4,
                                rotated_tile,
                            )),
                            Literal::positive(Variable::for_bottom_edge_color(x, y, color)),
                        );
                        for other_color in Color::iter() {
                            if other_color != Color::EXTERIOR && other_color != color {
                                // placed(x, y+1, rotated_tile) -> -bottom_edge_color(x, y, other_color)
                                clauses.push_binary(
                                    Literal::negative(Variable::for_tile_placement(
                                        x,
                                        y + 1_U4,
                                        rotated_tile,
                                    )),
                                    Literal::negative(Variable::for_bottom_edge_color(
                                        x,
                                        y,
                                        other_color,
                                    )),
                                );
                            }
                        }
                    } else {
                        // Can't place a gray edge in the middle of the board.
                        // -placed(x, y+1, rotated_tile)
                        clauses.push_unit(Literal::negative(Variable::for_tile_placement(
                            x,
                            y + 1_U4,
                            rotated_tile,
                        )));
                    }
                }
            }
        }
    }

    // Rule out top and bottom edges on the perimeter that aren't gray.
    for x in 0..16 {
        let x = U4::new_masked(x);
        for tile in Tile::values() {
            for rotation in Rotation::iter() {
                let rotated_tile = RotatedTile { tile, rotation };
                if rotated_tile.color(Side::Top) != Color::EXTERIOR {
                    clauses.push_unit(Literal::negative(Variable::for_tile_placement(
                        x,
                        0_U4,
                        rotated_tile,
                    )));
                }
                if rotated_tile.color(Side::Bottom) != Color::EXTERIOR {
                    clauses.push_unit(Literal::negative(Variable::for_tile_placement(
                        x,
                        15_U4,
                        rotated_tile,
                    )));
                }
            }
        }
    }

    // Rule out left and right edges on the perimeter that aren't gray.
    for y in 0..16 {
        let y = U4::new_masked(y);
        for tile in Tile::values() {
            for rotation in Rotation::iter() {
                let rotated_tile = RotatedTile { tile, rotation };
                if rotated_tile.color(Side::Left) != Color::EXTERIOR {
                    clauses.push_unit(Literal::negative(Variable::for_tile_placement(
                        0_U4,
                        y,
                        rotated_tile,
                    )));
                }
                if rotated_tile.color(Side::Right) != Color::EXTERIOR {
                    clauses.push_unit(Literal::negative(Variable::for_tile_placement(
                        15_U4,
                        y,
                        rotated_tile,
                    )));
                }
            }
        }
    }

    println!("p cnf {} {}", Variable::COUNT, clauses.len());
    clauses.print_dimacs_fragment(BufWriter::new(stdout().lock()))?;

    Ok(())
}
