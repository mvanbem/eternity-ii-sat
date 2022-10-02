use eternity_ii::{Color, Edge, RotatedTile, Rotation, Tile};

fn main() {
    let goal = std::env::args().skip(1).next().unwrap();
    println!("Searching for a tile with edge assignment {goal:?}");

    for tile in Tile::values() {
        for rotation in Rotation::VALUES {
            let rotated = RotatedTile { tile, rotation };
            if goal
                .chars()
                .map(Color::from_char)
                .zip(Edge::VALUES)
                .all(|(color, edge)| rotated.edge_color(edge) == color)
            {
                println!("Matched tile {} {}", tile.index(), rotation);
                return;
            }
        }
    }
    println!("No tile matched");
}
