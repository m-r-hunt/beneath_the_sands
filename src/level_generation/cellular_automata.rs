use super::{EnemyType, GeneratedLevel, StringErr, FLOOR, WALL};
use crate::physics::TileMap;
use crate::prelude::*;
use std::collections::HashMap;

const LEVEL_SIZE: i32 = 50;

fn r_n(level: &HashMap<(i32, i32), i32>, n: i32, (x, y): (i32, i32)) -> i32 {
    let mut total = 0;
    for dx in -n..=n {
        for dy in -n..=n {
            //if dx == 0 && dy == 0 { continue; }
            total += level.get(&(x + dx, y + dy)).cloned().unwrap_or(1);
        }
    }
    total
}

#[allow(clippy::cyclomatic_complexity)] // /me cries in professional
pub fn try_generate_level() -> Result<GeneratedLevel, StringErr> {
    let mut rng = rand::thread_rng();
    let mut level = HashMap::new();

    // Initialise randomly
    for y in 0..LEVEL_SIZE {
        for x in 0..LEVEL_SIZE {
            level.insert((x, y), if rng.gen_range(0, 100) < 40 { 1 } else { 0 });
        }
    }

    for _g in 0..4 {
        let mut next = HashMap::new();
        for y in 0..LEVEL_SIZE {
            for x in 0..LEVEL_SIZE {
                next.insert(
                    (x, y),
                    if r_n(&level, 1, (x, y)) >= 5 || r_n(&level, 2, (x, y)) <= 2 {
                        1
                    } else {
                        0
                    },
                );
            }
        }
        level = next;
    }
    for _g in 0..3 {
        let mut next = HashMap::new();
        for y in 0..LEVEL_SIZE {
            for x in 0..LEVEL_SIZE {
                next.insert((x, y), if r_n(&level, 1, (x, y)) >= 5 { 1 } else { 0 });
            }
        }
        level = next;
    }

    let mut tiles = HashMap::new();
    for x in -1..=LEVEL_SIZE {
        for y in -1..=LEVEL_SIZE {
            tiles.insert((x, y), WALL);
        }
    }

    for x in 0..LEVEL_SIZE {
        for y in 0..LEVEL_SIZE {
            if level[&(x, y)] == 1 {
                tiles.insert((x, y), WALL);
            } else {
                tiles.insert((x, y), FLOOR);
            }
        }
    }

    // TODO: Pick random quadrants for the start/exit.
    let mut start_position = (-1, -1);
    'outer: for x in 0..LEVEL_SIZE {
        for y in 0..=x {
            if level[&(x, y)] == 0 {
                start_position = (x, y);
                break 'outer;
            }
        }
    }

    let mut exit_position = (-1, -1);
    'outer2: for x in (0..LEVEL_SIZE).rev() {
        for y in (x..LEVEL_SIZE).rev() {
            if level[&(x, y)] == 0 {
                exit_position = (x, y);
                break 'outer2;
            }
        }
    }

    let mut chode_positions = Vec::new();
    for _ in 0..30 {
        let p = (
            rng.gen_range(0, LEVEL_SIZE),
            rng.gen_range(0, LEVEL_SIZE),
            if rng.gen_range(0.0, 1.0) > 0.8 {
                EnemyType::Shotgunner
            } else if rng.gen_range(0.0, 1.0) > 0.6 {
                EnemyType::Spinner
            } else {
                EnemyType::Chode
            },
        );
        if level[&(p.0, p.1)] == 0
            && ((p.0 - start_position.0).abs() >= 10 || (p.1 - start_position.1).abs() >= 10)
        {
            chode_positions.push(p);
        }
    }

    Ok(GeneratedLevel {
        tile_map: TileMap { tiles },
        start_position,
        exit_position,
        chode_positions,
    })
}
