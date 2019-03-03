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

fn manhatten_distance(from: (i32, i32), to: (i32, i32)) -> i32 {
    (from.0 - to.0).abs() + (from.1 - to.1).abs()
}

pub fn generate_level() -> GeneratedLevel {
    let mut rng = rand::thread_rng();
    let mut tile_map: TileMap = Default::default();
    // Assume start position is always 0, 0
    // Pick an end position
    let start = (0, 0);
    let end = (rng.gen_range(1, 5), rng.gen_range(0, 5));
    let mut path = vec![start];
    let mut current_pos = start;
    while current_pos != end {
        let mut choices = Vec::new();
        for choice in [(-1, 0), (1, 0), (0, -1), (0, 1)].iter() {
            if path
                .iter()
                .any(|c| *c == (current_pos.0 + choice.0, current_pos.1 + choice.1))
            {
                continue;
            }

            choices.push(choice);

            if manhatten_distance((current_pos.0 + choice.0, current_pos.1 + choice.1), end)
                < manhatten_distance(current_pos, end)
            {
                choices.push(choice);
            }
        }

        assert!(!choices.is_empty());

        let index = rng.gen_range(0, choices.len());
        let choice = choices[index];
        current_pos = (current_pos.0 + choice.0, current_pos.1 + choice.1);
        path.push(dbg!(current_pos));
    }

    while manhatten_distance(current_pos, start) != 1 {
        let mut choices = Vec::new();
        for choice in [(-1, 0), (1, 0), (0, -1), (0, 1)].iter() {
            if path
                .iter()
                .any(|c| *c == (current_pos.0 + choice.0, current_pos.1 + choice.1))
            {
                continue;
            }

            choices.push(choice);

            if manhatten_distance((current_pos.0 + choice.0, current_pos.1 + choice.1), start)
                < manhatten_distance(current_pos, start)
            {
                choices.push(choice);
            }
        }

        assert!(!choices.is_empty());

        let index = rng.gen_range(0, choices.len());
        let choice = choices[index];
        current_pos = (current_pos.0 + choice.0, current_pos.1 + choice.1);
        path.push(dbg!(current_pos));
    }
    path.push(start);

    for room in path.iter() {
        carve_room(*room, (10, 10), &mut tile_map);
    }
    for i in 0..path.len() - 1 {
        let dx = path[i + 1].0 - path[i].0;
        let dy = path[i + 1].1 - path[i].1;
        let steps = (dx * 20).abs().max((dy * 20).abs());
        for step in 0..steps {
            tile_map.tiles.insert(
                (path[i].0 * 20 + step * dx, path[i].1 * 20 + step * dy),
                Tile {
                    collision: false,
                    colour: rgba!(223, 201, 96, 1.0),
                },
            );
        }
    }
    GeneratedLevel {
        tile_map,
        start_position: (5, 5),
    }
}
