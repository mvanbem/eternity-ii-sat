use eternity_ii::{Color, RotatedTile, Rotation, Side, Tile};
use strum::IntoEnumIterator;

fn main() {
    let goal = std::env::args().skip(1).next().unwrap();
    println!("Searching for a tile with edge assignment {goal:?} (right, up, left, down)");

    for tile in Tile::values() {
        for rotation in Rotation::iter() {
            let oriented = RotatedTile { tile, rotation };
            if goal
                .chars()
                .map(Color::from_char)
                .map(Option::unwrap)
                .zip(Side::iter())
                .all(|(color, side)| oriented.color(side) == color)
            {
                println!("Matched tile {} {:?}", tile.to_primitive(), rotation);
                return;
            }
        }
    }
    println!("No tile matched");
}
