use bitvec::bitarr;
use rayon::prelude::{ParallelBridge, ParallelIterator};
use strum::IntoEnumIterator;

use crate::mosaic::{
    ArrayMosaic, RectangularMosaic, RotatedRectangularMosaic, RotatedSquareMosaic, SquareMosaic,
};
use crate::rectangular::{HorizontalSide, RectangularRotation};
use crate::set::builder::{SetBuilder, ShardBuilder};
use crate::set::rectangle::RectangularMosaicSet;
use crate::set::square::SquareMosaicSet;
use crate::{ExteriorMask, RotatedTile, Rotation, Side, Tile};

pub mod builder;
pub mod rectangle;
pub mod square;

/// Returns whether the rotated tile is a corner in canonical orientation.
///
/// ```text
/// ┌──────────┐
/// │ ┌────────┤
/// │ │╭─►     │
/// │ │ corner │
/// │ │     ◄─╯│
/// └─┴────────┘
/// ```
///
/// Corner pieces in canonical orientation have exterior sides on the top and
/// left.
fn is_canonical_corner(rotated_tile: RotatedTile) -> bool {
    rotated_tile.exterior_mask() == ExteriorMask::zero().with_top(true).with_left(true)
}

/// Returns whether the rotated tile is an edge in canonical orientation.
///
/// ```text
/// ┌─┬────────┐
/// │ │╭─►     │
/// │ │  edge  │
/// │ │     ◄─╯│
/// └─┴────────┘
/// ```
///
/// Edge pieces in canonical orientation have an exterior side on
/// the left.
fn is_canonical_edge(rotated_tile: RotatedTile) -> bool {
    rotated_tile.exterior_mask() == ExteriorMask::zero().with_left(true)
}

/// Returns whether the rotated tile is a center in canonical orientation.
///
/// ```text
/// ┌────────┐
/// │ ▴    ▴ │
/// │ center │
/// │        │
/// └────────┘
/// ```
///
/// Center pieces have no exterior sides. The identity rotation is canonical.
fn is_canonical_center(rotated_tile: RotatedTile) -> bool {
    rotated_tile.exterior_mask() == ExteriorMask::zero()
        && rotated_tile.rotation == Rotation::Identity
}

pub fn build_1x1_sets() -> (
    SquareMosaicSet<1, impl SquareMosaic<1>>,
    SquareMosaicSet<1, impl SquareMosaic<1>>,
    SquareMosaicSet<1, impl SquareMosaic<1>>,
) {
    // Consider all possible rotated tiles, classifying them and collecting only
    // the canonical ones.
    let mut square_1x1_corners = SquareMosaicSet::new();
    let mut square_1x1_edges = SquareMosaicSet::new();
    let mut square_1x1_centers = SquareMosaicSet::new();
    for tile_id in 0..=255 {
        let tile = Tile::from_primitive(tile_id);
        for rotation in Rotation::iter() {
            let rotated_tile = RotatedTile { tile, rotation };
            if is_canonical_corner(rotated_tile) {
                square_1x1_corners.insert(mosaic![[@rotated_tile]]);
            } else if is_canonical_edge(rotated_tile) {
                square_1x1_edges.insert(mosaic![[@rotated_tile]]);
            } else if is_canonical_center(rotated_tile) {
                square_1x1_centers.insert(mosaic![[@rotated_tile]]);
            }
        }
    }
    (square_1x1_corners, square_1x1_edges, square_1x1_centers)
}

pub fn build_1x1_sets_with_clues() -> (
    SquareMosaicSet<1, impl SquareMosaic<1>>,
    SquareMosaicSet<1, impl SquareMosaic<1>>,
    SquareMosaicSet<1, impl SquareMosaic<1>>,
    SquareMosaicSet<1, impl SquareMosaic<1>>,
    SquareMosaicSet<1, impl SquareMosaic<1>>,
    SquareMosaicSet<1, impl SquareMosaic<1>>,
    SquareMosaicSet<1, impl SquareMosaic<1>>,
    SquareMosaicSet<1, impl SquareMosaic<1>>,
) {
    let mut square_1x1_centers_c3_clue = SquareMosaicSet::new();
    square_1x1_centers_c3_clue.insert(mosaic!([76 Identity]));
    let mut square_1x1_centers_c14_clue = SquareMosaicSet::new();
    square_1x1_centers_c14_clue.insert(mosaic!([179 QuarterTurnLeft]));
    let mut square_1x1_centers_i8_clue = SquareMosaicSet::new();
    square_1x1_centers_i8_clue.insert(mosaic!([135 Identity]));
    let mut square_1x1_centers_n3_clue = SquareMosaicSet::new();
    square_1x1_centers_n3_clue.insert(mosaic!([211 HalfTurn]));
    let mut square_1x1_centers_n14_clue = SquareMosaicSet::new();
    square_1x1_centers_n14_clue.insert(mosaic!([125 QuarterTurnRight]));

    // Consider all possible rotated tiles, classifying them and collecting only
    // the canonical ones that are not clue tiles.
    let mut square_1x1_corners_no_clues = SquareMosaicSet::new();
    let mut square_1x1_edges_no_clues = SquareMosaicSet::new();
    let mut square_1x1_centers_no_clues = SquareMosaicSet::new();
    for tile_id in 0..=255 {
        // Skip the clue tiles.
        if [76, 125, 135, 179, 211].contains(&tile_id) {
            continue;
        }
        let tile = Tile::from_primitive(tile_id);
        for rotation in Rotation::iter() {
            let rotated_tile = RotatedTile { tile, rotation };
            if is_canonical_corner(rotated_tile) {
                square_1x1_corners_no_clues.insert(mosaic![[@rotated_tile]]);
            } else if is_canonical_edge(rotated_tile) {
                square_1x1_edges_no_clues.insert(mosaic![[@rotated_tile]]);
            } else if is_canonical_center(rotated_tile) {
                square_1x1_centers_no_clues.insert(mosaic![[@rotated_tile]]);
            }
        }
    }
    (
        square_1x1_corners_no_clues,
        square_1x1_edges_no_clues,
        square_1x1_centers_no_clues,
        square_1x1_centers_c3_clue,
        square_1x1_centers_c14_clue,
        square_1x1_centers_i8_clue,
        square_1x1_centers_n3_clue,
        square_1x1_centers_n14_clue,
    )
}

fn combine_squares_horizontally_to_rectangle<const SHORT: usize, const LONG: usize>(
    a: impl SquareMosaic<SHORT>,
    b: impl SquareMosaic<SHORT>,
) -> Option<ArrayMosaic<LONG, SHORT>> {
    assert_eq!(SHORT * 2, LONG);

    let mut used_tiles = bitarr![0; 256];
    let mut mark = |rotated_tile: RotatedTile| -> Option<RotatedTile> {
        let index = rotated_tile.tile.to_primitive() as usize;
        if used_tiles[index] {
            return None;
        }
        used_tiles.set(index, true);
        Some(rotated_tile)
    };

    let mut mosaic = ArrayMosaic {
        tiles: [[RotatedTile::ZERO; LONG]; SHORT],
    };
    for y in 0..SHORT {
        for x in 0..SHORT {
            mosaic.tiles[y][x] = mark(a.get(x, y))?;
            mosaic.tiles[y][x + SHORT] = mark(b.get(x, y))?;
        }
    }
    Some(mosaic)
}

fn combine_rectangles_vertically_to_square<const SHORT: usize, const LONG: usize>(
    a: impl RectangularMosaic<LONG, SHORT>,
    b: impl RectangularMosaic<LONG, SHORT>,
) -> Option<ArrayMosaic<LONG, LONG>> {
    assert_eq!(SHORT * 2, LONG);

    let mut used_tiles = bitarr![0; 256];
    let mut mark = |rotated_tile: RotatedTile| -> Option<RotatedTile> {
        let index = rotated_tile.tile.to_primitive() as usize;
        if used_tiles[index] {
            return None;
        }
        used_tiles.set(index, true);
        Some(rotated_tile)
    };

    let mut mosaic = ArrayMosaic {
        tiles: [[RotatedTile::ZERO; LONG]; LONG],
    };
    for y in 0..SHORT {
        for x in 0..LONG {
            mosaic.tiles[y][x] = mark(a.get(x, y))?;
            mosaic.tiles[y + SHORT][x] = mark(b.get(x, y))?;
        }
    }
    Some(mosaic)
}

pub fn min_rotated_tile<const W: usize, const H: usize>(
    mosaic: impl RectangularMosaic<W, H>,
) -> RotatedTile {
    let mut result = RotatedTile::MAX;
    for y in 0..mosaic.height() {
        for x in 0..mosaic.width() {
            result = result.min(mosaic.get(x, y));
        }
    }
    result
}

pub fn build_rectangles_memo<
    const SHORT: usize,
    const LONG: usize,
    B: SetBuilder<Item = ArrayMosaic<LONG, SHORT>>,
    MosaicA: SquareMosaic<SHORT> + Send + Sync,
    MosaicB: SquareMosaic<SHORT> + Send + Sync,
    Memo,
>(
    mut set_builder: B,
    a_set: &SquareMosaicSet<SHORT, MosaicA>,
    a_memo: impl Fn(RotatedSquareMosaic<SHORT, MosaicA>) -> Option<Memo> + Send + Sync,
    b_set: &SquareMosaicSet<SHORT, MosaicB>,
    b_filter: impl Fn(&Memo, RotatedSquareMosaic<SHORT, MosaicB>) -> bool + Send + Sync,
) -> B::Result {
    assert_eq!(LONG, SHORT * 2);

    a_set.iter_by_edge(Side::Right).par_bridge().for_each_with(
        set_builder.new_shard(),
        |shard_builder, (a_shared_edge, a_set)| {
            let b_shared_edge = a_shared_edge.reversed();
            for (_, a) in a_set {
                if let Some(memo) = a_memo(a) {
                    for (_, b) in b_set.query(Side::Left, &b_shared_edge) {
                        if b_filter(&memo, b) {
                            if let Some(mosaic) = combine_squares_horizontally_to_rectangle(a, b) {
                                shard_builder.insert(mosaic);
                            }
                        }
                    }
                }
            }
        },
    );
    set_builder.finish()
}

pub fn build_rectangles<
    const SHORT: usize,
    const LONG: usize,
    B: SetBuilder<Item = ArrayMosaic<LONG, SHORT>>,
    MosaicA: SquareMosaic<SHORT> + Send + Sync,
    MosaicB: SquareMosaic<SHORT> + Send + Sync,
>(
    set_builder: B,
    a_set: &SquareMosaicSet<SHORT, MosaicA>,
    a_filter: impl Fn(RotatedSquareMosaic<SHORT, MosaicA>) -> bool + Send + Sync,
    b_set: &SquareMosaicSet<SHORT, MosaicB>,
    b_filter: impl Fn(RotatedSquareMosaic<SHORT, MosaicB>) -> bool + Send + Sync,
) -> B::Result {
    build_rectangles_memo(
        set_builder,
        a_set,
        |a| if a_filter(a) { Some(()) } else { None },
        b_set,
        |(), b| b_filter(b),
    )
}

pub fn build_squares_memo<
    const SHORT: usize,
    const LONG: usize,
    B: SetBuilder<Item = ArrayMosaic<LONG, LONG>>,
    MosaicA: RectangularMosaic<LONG, SHORT> + Send + Sync,
    MosaicB: RectangularMosaic<LONG, SHORT> + Send + Sync,
    Memo,
>(
    mut set_builder: B,
    a_set: &RectangularMosaicSet<LONG, SHORT, MosaicA>,
    a_memo: impl Fn(RotatedRectangularMosaic<LONG, SHORT, MosaicA>) -> Option<Memo> + Send + Sync,
    b_set: &RectangularMosaicSet<LONG, SHORT, MosaicB>,
    b_filter: impl Fn(&Memo, RotatedRectangularMosaic<LONG, SHORT, MosaicB>) -> bool + Send + Sync,
) -> B::Result {
    assert_eq!(LONG, SHORT * 2);

    a_set
        .iter_by_horizontal_edge(HorizontalSide::Bottom)
        .par_bridge()
        .for_each_with(
            set_builder.new_shard(),
            |shard_builder, (a_shared_edge, a_set)| {
                let b_shared_edge = a_shared_edge.reversed();
                for (_, a) in a_set {
                    if let Some(memo) = a_memo(a) {
                        for (_, b) in b_set.query_horizontal(HorizontalSide::Top, &b_shared_edge) {
                            if b_filter(&memo, b) {
                                if let Some(mosaic) = combine_rectangles_vertically_to_square(a, b)
                                {
                                    shard_builder.insert(mosaic);
                                }
                            }
                        }
                    }
                }
            },
        );
    set_builder.finish()
}

pub fn build_squares<
    const SHORT: usize,
    const LONG: usize,
    B: SetBuilder<Item = ArrayMosaic<LONG, LONG>>,
    MosaicA: RectangularMosaic<LONG, SHORT> + Send + Sync,
    MosaicB: RectangularMosaic<LONG, SHORT> + Send + Sync,
>(
    set_builder: B,
    a_set: &RectangularMosaicSet<LONG, SHORT, MosaicA>,
    a_filter: impl Fn(RotatedRectangularMosaic<LONG, SHORT, MosaicA>) -> bool + Send + Sync,
    b_set: &RectangularMosaicSet<LONG, SHORT, MosaicB>,
    b_filter: impl Fn(RotatedRectangularMosaic<LONG, SHORT, MosaicB>) -> bool + Send + Sync,
) -> B::Result {
    build_squares_memo(
        set_builder,
        a_set,
        |a| if a_filter(a) { Some(()) } else { None },
        b_set,
        |(), b| b_filter(b),
    )
}

/// Builds a rectangular corner mosaic set from smaller square corner and edge
/// mosaic sets.
///
/// ```text
/// ┌──────────┬────────┐      ┌───────────────────┐
/// │ ┌────────┼────────┤      │ ┌─────────────────┤
/// │ │ ▴    ▴ │      ▸ │      │ │ ▴             ▴ │
/// │ │ corner │  edge  │  =>  │ │     corner      │
/// │ │        │      ▸ │      │ │                 │
/// └─┴────────┴────────┘      └─┴─────────────────┘
/// ```
///
/// - The square corner mosaic is in canonical orientation.
/// - The square edge mosaic is rotated a quarter turn right from canonical
///   orientation.
/// - The resulting rectangular corner mosaic is in canonical orientation.
pub fn build_rectangular_corners<
    const SHORT: usize,
    const LONG: usize,
    B: SetBuilder<Item = ArrayMosaic<LONG, SHORT>>,
    CornerMosaic: SquareMosaic<SHORT> + Send + Sync,
    EdgeMosaic: SquareMosaic<SHORT> + Send + Sync,
>(
    set_builder: B,
    corners: &SquareMosaicSet<SHORT, CornerMosaic>,
    edges: &SquareMosaicSet<SHORT, EdgeMosaic>,
) -> B::Result {
    build_rectangles(
        set_builder,
        corners,
        |a| a.rotation == Rotation::Identity,
        edges,
        |b| b.rotation == Rotation::QuarterTurnRight,
    )
}

/// Builds a rectangular edge mosaic set from smaller square edge and center
/// mosaic sets.
///
/// ```text
/// ┌─┬────────┬────────┐      ┌─┬─────────────────┐
/// │ │ ▴    ▴ │╭─►     │      │ │ ▴             ▴ │
/// │ │  edge  │ center │  =>  │ │      edge       │
/// │ │        │     ◄─╯│      │ │                 │
/// └─┴────────┴────────┘      └─┴─────────────────┘
/// ```
///
/// - The square edge mosaic is in canonical orientation.
/// - The square center mosaic is in an arbitrary orientation.
/// - The resulting rectangular edge mosaic is in canonical orientation.
pub fn build_rectangular_edges<
    const SHORT: usize,
    const LONG: usize,
    B: SetBuilder<Item = ArrayMosaic<LONG, SHORT>>,
    EdgeMosaic: SquareMosaic<SHORT> + Send + Sync,
    CenterMosaic: SquareMosaic<SHORT> + Send + Sync,
>(
    set_builder: B,
    edges: &SquareMosaicSet<SHORT, EdgeMosaic>,
    centers: &SquareMosaicSet<SHORT, CenterMosaic>,
) -> B::Result {
    build_rectangles(
        set_builder,
        edges,
        |a| a.rotation == Rotation::Identity,
        centers,
        |_b| true,
    )
}

/// Builds a rectangular center mosaic set from a smaller square center mosaic
/// set.
///
/// ```text
/// ┌────────┬────────┐      ┌─────────────────┐
/// │╭─►     │╭─►     │      │ ▴             ▴ │
/// │ center │ center │  =>  │     center      │
/// │     ◄─╯│     ◄─╯│      │                 │
/// └────────┴────────┘      └─────────────────┘
/// ```
///
/// - Each square center mosaic is in an arbitrary orientation.
/// - The resulting rectangular center mosaic is in canonical orientation only
///   if its lowest-numbered tile is in its canonical orientation or is rotated
///   a quarter turn left from canonical orientation.
pub fn build_rectangular_centers<
    const SHORT: usize,
    const LONG: usize,
    B: SetBuilder<Item = ArrayMosaic<LONG, SHORT>>,
    CenterMosaic: SquareMosaic<SHORT> + Send + Sync,
>(
    set_builder: B,
    centers: &SquareMosaicSet<SHORT, CenterMosaic>,
) -> B::Result {
    build_rectangles_memo(
        set_builder,
        centers,
        |a| Some(min_rotated_tile(a)),
        centers,
        |&a_min_rotated_tile, b| {
            let min_rotated_tile = a_min_rotated_tile.min(min_rotated_tile(b));
            min_rotated_tile.rotation == Rotation::Identity
                || min_rotated_tile.rotation == Rotation::QuarterTurnLeft
        },
    )
}

/// Builds a square corner mosaic set from smaller rectangular corner and edge
/// mosaic sets.
///
/// ```text
/// ┌───────────────────┐      ┌───────────────────┐
/// │ ┌─────────────────┤      │ ┌─────────────────┤
/// │ │ ▴             ▴ │      │ │ ▴             ▴ │
/// │ │     corner      │      │ │                 │
/// │ │                 │      │ │                 │
/// ├─┼─────────────────┤  =>  │ │     corner      │
/// │ │ ▴             ▴ │      │ │                 │
/// │ │      edge       │      │ │                 │
/// │ │                 │      │ │                 │
/// └─┴─────────────────┘      └─┴─────────────────┘
/// ```
///
/// - The rectangular corner mosaic is in canonical orientation.
/// - The rectangular edge mosaic is in canonical orientation.
/// - The resulting square corner mosaic is in canonical orientation.
pub fn build_square_corners<
    const SHORT: usize,
    const LONG: usize,
    B: SetBuilder<Item = ArrayMosaic<LONG, LONG>>,
    CornerMosaic: RectangularMosaic<LONG, SHORT> + Send + Sync,
    EdgeMosaic: RectangularMosaic<LONG, SHORT> + Send + Sync,
>(
    set_builder: B,
    corners: &RectangularMosaicSet<LONG, SHORT, CornerMosaic>,
    edges: &RectangularMosaicSet<LONG, SHORT, EdgeMosaic>,
) -> B::Result {
    build_squares(
        set_builder,
        corners,
        |a| a.rotation == RectangularRotation::Identity,
        edges,
        |b| b.rotation == RectangularRotation::Identity,
    )
}

/// Builds a square edge mosaic set from a smaller rectangular edge mosaic set.
///
/// ```text
/// ┌─┬─────────────────┐      ┌─┬─────────────────┐
/// │ │ ▴             ▴ │      │ │ ▴             ▴ │
/// │ │      edge       │      │ │                 │
/// │ │                 │      │ │                 │
/// ├─┼─────────────────┤  =>  │ │      edge       │
/// │ │ ▴             ▴ │      │ │                 │
/// │ │      edge       │      │ │                 │
/// │ │                 │      │ │                 │
/// └─┴─────────────────┘      └─┴─────────────────┘
/// ```
///
/// - Both rectangular edge mosaics are in canonical orientation.
/// - The resulting square edge mosaic is in canonical orientation.
pub fn build_square_edges<
    const SHORT: usize,
    const LONG: usize,
    B: SetBuilder<Item = ArrayMosaic<LONG, LONG>>,
    EdgeMosaic: RectangularMosaic<LONG, SHORT> + Send + Sync,
>(
    set_builder: B,
    edges: &RectangularMosaicSet<LONG, SHORT, EdgeMosaic>,
) -> B::Result {
    build_squares(
        set_builder,
        edges,
        |a| a.rotation == RectangularRotation::Identity,
        edges,
        |b| b.rotation == RectangularRotation::Identity,
    )
}

/// Builds a square center mosaic set from a smaller rectangular center mosaic
/// set.
///
/// ```text
/// ┌─────────────────┐      ┌─────────────────┐
/// │╭─►              │      │ ▴             ▴ │
/// │     center      │      │                 │
/// │              ◄─╯│      │                 │
/// ├─────────────────┤  =>  │     center      │
/// │╭─►              │      │                 │
/// │     center      │      │                 │
/// │              ◄─╯│      │                 │
/// └─────────────────┘      └─────────────────┘
/// ```
///
/// - Each rectangular center mosaic is in an arbitrary orientation.
/// - The resulting square center mosaic is in canonical orientation only if its
///   lowest-numbered tile is in its canonical orientation.
pub fn build_square_centers<
    const SHORT: usize,
    const LONG: usize,
    B: SetBuilder<Item = ArrayMosaic<LONG, LONG>>,
    CenterMosaic: RectangularMosaic<LONG, SHORT> + Send + Sync,
>(
    set_builder: B,
    centers: &RectangularMosaicSet<LONG, SHORT, CenterMosaic>,
) -> B::Result {
    build_squares_memo(
        set_builder,
        centers,
        |a| Some(min_rotated_tile(a)),
        centers,
        |&a_min_rotated_tile, b| {
            let min_rotated_tile = a_min_rotated_tile.min(min_rotated_tile(b));
            min_rotated_tile.rotation == Rotation::Identity
        },
    )
}
