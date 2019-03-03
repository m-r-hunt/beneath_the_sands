use crate::physics::{Tile, TileMap};
use rand::Rng;

// # S-#-#-#
//   =   o |
// # #-# #-#
//     | |
// # # E-# #

pub struct GeneratedLevel {
    pub tile_map: TileMap,
    pub start_position: (i32, i32),
}

fn carve_room(position: (i32, i32), size: (i32, i32), tile_map: &mut TileMap) {
    for x in -1..=size.0 {
        for y in -1..=size.1 {
            tile_map.tiles.insert(
                (position.0 * 20 + x, position.1 * 20 + y),
                Tile {
                    collision: true,
                    colour: rgba!(128, 128, 128, 1.0),
                },
            );
        }
    }
    for x in 0..size.0 {
        for y in 0..size.1 {
            tile_map.tiles.insert(
                (position.0 * 20 + x, position.1 * 20 + y),
                Tile {
                    collision: false,
                    colour: rgba!(223, 201, 96, 1.0),
                },
            );
        }
    }
}

pub fn generate_level() -> GeneratedLevel {
    let mut rng = rand::thread_rng();
    let mut tile_map: TileMap = Default::default();
    // Assume start position is always 0, 0
    // Pick an end position
    let end = (rng.gen_range(1, 5), rng.gen_range(0, 5));
    carve_room((0, 0), (10, 10), &mut tile_map);
    carve_room(end, (10, 10), &mut tile_map);
    GeneratedLevel {
        tile_map,
        start_position: (5, 5),
    }
}
