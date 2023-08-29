use std::io::{stdin, BufRead, BufReader};

use anyhow::Result;
use bitint::prelude::*;
use eternity_ii::sat::{Variable, VariableKind};
use eternity_ii::Side;
use strum::IntoEnumIterator;

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

#[bitint_literals]
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
                    if used_tiles[rotated_tile.tile.to_primitive() as usize] {
                        log_error!(v, "Tile {:?} used more than once", rotated_tile.tile);
                    }
                    used_tiles[rotated_tile.tile.to_primitive() as usize] = true;
                    for side in Side::iter() {
                        board_edges[index(x, y, side)] = rotated_tile.color(side).to_byte_char();
                    }
                }
                VariableKind::RightEdgeColor { x, y, color } => {
                    if board_edges[index(x, y, Side::Right)] != color.to_byte_char() {
                        log_warning!(v, "Conflict at ({}, {}) right edge", x, y);
                    }
                    if board_edges[index(x + 1_U4, y, Side::Left)] != color.to_byte_char() {
                        log_warning!(v, "Conflict at ({}, {}) left edge", x + 1_U4, y);
                    }
                }
                VariableKind::BottomEdgeColor { x, y, color } => {
                    if board_edges[index(x, y, Side::Bottom)] != color.to_byte_char() {
                        log_warning!(v, "Conflict at ({}, {}) bottom edge", x, y);
                    }
                    if board_edges[index(x, y + 1_U4, Side::Top)] != color.to_byte_char() {
                        log_warning!(v, "Conflict at ({}, {}) top edge", x, y + 1_U4);
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

fn index(x: U4, y: U4, side: Side) -> usize {
    64 * y.to_primitive() as usize
        + 4 * x.to_primitive() as usize
        + match side {
            Side::Top => 0,
            Side::Right => 1,
            Side::Bottom => 2,
            Side::Left => 3,
        }
}
