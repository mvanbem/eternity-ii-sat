use std::io::{stdin, BufRead, BufReader};

use anyhow::Result;
use eternity_ii::{Edge, Variable, VariableKind};

fn main() -> Result<()> {
    let mut literals = Vec::new();
    'outer: for line in BufReader::new(stdin()).lines() {
        if let Some(line) = line?.strip_prefix('v') {
            for literal in line.split_ascii_whitespace() {
                if !literal.is_empty() {
                    let literal = isize::from_str_radix(literal, 10)?;
                    if literal == 0 {
                        break 'outer;
                    }
                    literals.push(literal);
                }
            }
        }
    }

    // Put tile placements before edge colors so we can assert below.
    literals.sort_by_key(|literal| literal.abs());

    let mut used_tiles = [false; 256];
    let mut board_edges = [b'_'; 1024];
    for literal in literals {
        if literal > 0 {
            match Variable::from(literal.abs() as usize).kind() {
                VariableKind::TilePlacement { x, y, rotated_tile } => {
                    assert!(!used_tiles[rotated_tile.tile.index()]);
                    used_tiles[rotated_tile.tile.index()] = true;
                    for edge in Edge::VALUES {
                        board_edges[64 * y + 4 * x + edge.index()] =
                            rotated_tile.edge_color(edge).as_u8();
                    }
                }
                VariableKind::RightEdgeColor { x, y, color } => {
                    assert_eq!(
                        board_edges[64 * y + 4 * x + Edge::RIGHT.index()],
                        color.as_u8(),
                    );
                    assert_eq!(
                        board_edges[64 * y + 4 * (x + 1) + Edge::LEFT.index()],
                        color.as_u8(),
                    );
                }
                VariableKind::BottomEdgeColor { x, y, color } => {
                    assert_eq!(
                        board_edges[64 * y + 4 * x + Edge::BOTTOM.index()],
                        color.as_u8(),
                    );
                    assert_eq!(
                        board_edges[64 * (y + 1) + 4 * x + Edge::TOP.index()],
                        color.as_u8(),
                    );
                }
            }
        }
    }
    assert_eq!(used_tiles, [true; 256]);

    println!(
        "https://e2.bucas.name/#board_w=16&board_h=16&board_edges={}&motifs_order=jblackwood",
        std::str::from_utf8(&board_edges)?,
    );

    Ok(())
}
