use std::io::{stdout, BufWriter};

use anyhow::Result;
use eternity_ii::sat::{Clauses, Literal};
use eternity_ii::{Color, Edge, RotatedTile, Rotation, Tile, Variable};

fn main() -> Result<()> {
    let mut clauses = Clauses::default();

    // One rotated tile per cell.
    for y in 0..16 {
        for x in 0..16 {
            let mut variables = Vec::new();
            for tile in Tile::values() {
                for rotation in Rotation::VALUES {
                    variables.push(Variable::for_tile_placement(
                        x,
                        y,
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
                for rotation in Rotation::VALUES {
                    variables.push(Variable::for_tile_placement(
                        x,
                        y,
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
        for x in 0..15 {
            for tile in Tile::values() {
                for rotation in Rotation::VALUES {
                    let rotated_tile = RotatedTile { tile, rotation };

                    // Right edge of the tile at (x, y).
                    let color = rotated_tile.edge_color(Edge::RIGHT);
                    if color != Color::GRAY {
                        // placed(x, y, rotated_tile) -> right_edge_color(x, y, color)
                        clauses.push_binary(
                            Literal::negative(Variable::for_tile_placement(x, y, rotated_tile)),
                            Literal::positive(Variable::for_right_edge_color(x, y, color)),
                        );
                        for other_color in Color::values() {
                            if other_color != Color::GRAY && other_color != color {
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
                    let color = rotated_tile.edge_color(Edge::LEFT);
                    if color != Color::GRAY {
                        // placed(x+1, y, rotated_tile) -> right_edge_color(x, y, color)
                        clauses.push_binary(
                            Literal::negative(Variable::for_tile_placement(x + 1, y, rotated_tile)),
                            Literal::positive(Variable::for_right_edge_color(x, y, color)),
                        );
                        for other_color in Color::values() {
                            if other_color != Color::GRAY && other_color != color {
                                // placed(x+1, y, rotated_tile) -> -right_edge_color(x, y, other_color)
                                clauses.push_binary(
                                    Literal::negative(Variable::for_tile_placement(
                                        x + 1,
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
                            x + 1,
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
        for x in 0..16 {
            for tile in Tile::values() {
                for rotation in Rotation::VALUES {
                    let rotated_tile = RotatedTile { tile, rotation };

                    // Bottom edge of the tile at (x, y).
                    let color = rotated_tile.edge_color(Edge::BOTTOM);
                    if color != Color::GRAY {
                        // placed(x, y, rotated_tile) -> bottom_edge_color(x, y, color)
                        clauses.push_binary(
                            Literal::negative(Variable::for_tile_placement(x, y, rotated_tile)),
                            Literal::positive(Variable::for_bottom_edge_color(x, y, color)),
                        );
                        for other_color in Color::values() {
                            if other_color != Color::GRAY && other_color != color {
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
                    let color = rotated_tile.edge_color(Edge::TOP);
                    if color != Color::GRAY {
                        // placed(x, y+1, rotated_tile) -> bottom_edge_color(x, y, color)
                        clauses.push_binary(
                            Literal::negative(Variable::for_tile_placement(x, y + 1, rotated_tile)),
                            Literal::positive(Variable::for_bottom_edge_color(x, y, color)),
                        );
                        for other_color in Color::values() {
                            if other_color != Color::GRAY && other_color != color {
                                // placed(x, y+1, rotated_tile) -> -bottom_edge_color(x, y, other_color)
                                clauses.push_binary(
                                    Literal::negative(Variable::for_tile_placement(
                                        x,
                                        y + 1,
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
                            y + 1,
                            rotated_tile,
                        )));
                    }
                }
            }
        }
    }

    // Rule out top and bottom edges on the perimeter that aren't gray.
    for x in 0..16 {
        for tile in Tile::values() {
            for rotation in Rotation::VALUES {
                let rotated_tile = RotatedTile { tile, rotation };
                if rotated_tile.edge_color(Edge::TOP) != Color::GRAY {
                    clauses.push_unit(Literal::negative(Variable::for_tile_placement(
                        x,
                        0,
                        rotated_tile,
                    )));
                }
                if rotated_tile.edge_color(Edge::BOTTOM) != Color::GRAY {
                    clauses.push_unit(Literal::negative(Variable::for_tile_placement(
                        x,
                        15,
                        rotated_tile,
                    )));
                }
            }
        }
    }

    // Rule out left and right edges on the perimeter that aren't gray.
    for y in 0..16 {
        for tile in Tile::values() {
            for rotation in Rotation::VALUES {
                let rotated_tile = RotatedTile { tile, rotation };
                if rotated_tile.edge_color(Edge::LEFT) != Color::GRAY {
                    clauses.push_unit(Literal::negative(Variable::for_tile_placement(
                        0,
                        y,
                        rotated_tile,
                    )));
                }
                if rotated_tile.edge_color(Edge::RIGHT) != Color::GRAY {
                    clauses.push_unit(Literal::negative(Variable::for_tile_placement(
                        15,
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
