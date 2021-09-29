use std::io::{stdin, BufRead, BufReader};

use anyhow::Result;
use eternity_ii::{Edge, Variable, VariableKind};

use crate::validation::Validation;

mod validation {
    use anyhow::{anyhow, Result};

    #[derive(Default)]
    pub struct Validation {
        error_count: usize,
        warning_count: usize,
    }

    impl Validation {
        pub fn log_error(&mut self, msg: &str) {
            self.error_count += 1;
            eprintln!("ERROR: {}", msg);
        }

        pub fn log_warning(&mut self, msg: &str) {
            self.warning_count += 1;
            eprintln!("WARNING: {}", msg);
        }

        pub fn finish(self) -> Result<()> {
            eprintln!("Warning count: {}", self.warning_count);
            if self.error_count == 0 {
                Ok(())
            } else {
                Err(anyhow!("Error count: {}", self.error_count))
            }
        }
    }
}

macro_rules! log_error {
    ($v:ident, $($args:tt)*) => {
        $v.log_error(&format!($($args)*));
    };
}

macro_rules! log_warning {
    ($v:ident, $($args:tt)*) => {
        $v.log_warning(&format!($($args)*));
    };
}

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

    // Put tile placements before edge colors so we can validate as we go.
    literals.sort_by_key(|literal| literal.abs());

    let mut v = Validation::default();

    let mut used_tiles = [false; 256];
    let mut board_edges = [b'_'; 1024];
    for literal in literals {
        if literal > 0 {
            match Variable::from(literal.abs() as usize).kind() {
                VariableKind::TilePlacement { x, y, rotated_tile } => {
                    if used_tiles[rotated_tile.tile.index()] {
                        log_error!(v, "Tile {} used more than once", rotated_tile.tile.index());
                    }
                    used_tiles[rotated_tile.tile.index()] = true;
                    for edge in Edge::VALUES {
                        board_edges[64 * y + 4 * x + edge.index()] =
                            rotated_tile.edge_color(edge).as_u8();
                    }
                }
                VariableKind::RightEdgeColor { x, y, color } => {
                    if board_edges[64 * y + 4 * x + Edge::RIGHT.index()] != color.as_u8() {
                        log_warning!(v, "Conflict at ({}, {}) right edge", x, y);
                    }
                    if board_edges[64 * y + 4 * (x + 1) + Edge::LEFT.index()] != color.as_u8() {
                        log_warning!(v, "Conflict at ({}, {}) left edge", x + 1, y);
                    }
                }
                VariableKind::BottomEdgeColor { x, y, color } => {
                    if board_edges[64 * y + 4 * x + Edge::BOTTOM.index()] != color.as_u8() {
                        log_warning!(v, "Conflict at ({}, {}) bottom edge", x, y);
                    }
                    if board_edges[64 * (y + 1) + 4 * x + Edge::TOP.index()] != color.as_u8() {
                        log_warning!(v, "Conflict at ({}, {}) top edge", x, y + 1);
                    }
                }
            }
        }
    }

    for tile_index in 0..256 {
        if !used_tiles[tile_index] {
            log_error!(v, "Tile {} not used", tile_index);
        }
    }

    println!(
        "https://e2.bucas.name/#board_w=16&board_h=16&board_edges={}&motifs_order=jblackwood",
        std::str::from_utf8(&board_edges)?,
    );

    v.finish()
}
