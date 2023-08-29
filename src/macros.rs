#[macro_export]
macro_rules! rotated_tile {
    ($tile_id:literal) => {
        $crate::RotatedTile {
            tile: $crate::Tile::from_primitive($tile_id),
            rotation: $crate::Rotation::Identity,
        }
    };
    ($tile_id:literal $rotation:ident) => {
        $crate::RotatedTile {
            tile: $crate::Tile::from_primitive($tile_id),
            rotation: $crate::Rotation::$rotation,
        }
    };
    (@$rotated_tile:expr) => {
        $rotated_tile
    };
}

#[macro_export]
macro_rules! mosaic {
    ($( [$( $( $tile_id:literal )? $( $rotation:ident )? $( @$rotated_tile:expr )? ),+] ),+) => {
        $crate::mosaic::ArrayMosaic {
            tiles: [$( [$(
                $crate::rotated_tile!($( $tile_id )? $( $rotation )? $( @$rotated_tile )?)
            ),+] ),+],
        }
    };
}

#[cfg(test)]
#[macro_export]
macro_rules! btree_map {
    ($( $key:expr => $value:expr ),* $( , )?) => {{
        let mut result = ::std::collections::BTreeMap::new();
        $( result.insert($key, $value); )*
        result
    }}
}

#[cfg(test)]
#[macro_export]
macro_rules! btree_set {
    ($( $item:expr ),* $( , )?) => {{
        let mut result = ::std::collections::BTreeSet::new();
        $( result.insert($item); )*
        result
    }}
}
