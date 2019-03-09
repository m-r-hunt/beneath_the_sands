use crate::physics::{Tile, TileMap};

const WALL: Tile = Tile {
    collision: true,
    colour: rgba!(128, 128, 128, 1.0),
};

const FLOOR: Tile = Tile {
    collision: false,
    colour: rgba!(223, 201, 96, 1.0),
};

pub const BOSS_ARENA_SIZE: i32 = 10;

#[derive(Copy, Clone, Debug)]
pub enum LevelStyle {
    Cyclic,
    CellularAutomata,
}

pub fn generate_level(style: LevelStyle) -> GeneratedLevel {
    let gen_fn = match style {
        LevelStyle::Cyclic => cyclic::try_generate_level,
        LevelStyle::CellularAutomata => cellular_automata::try_generate_level,
    };
    loop {
        match gen_fn() {
            Ok(l) => return l,
            Err(e) => {
                dbg!(e);
            }
        }
    }
}

pub enum EnemyType {
    Chode,
    Shotgunner,
    Spinner,
}

pub struct GeneratedLevel {
    pub tile_map: TileMap,
    pub start_position: (i32, i32),
    pub exit_position: (i32, i32),
    pub chode_positions: Vec<(i32, i32, EnemyType)>,
}

#[derive(Debug)]
pub struct StringErr(String);

mod cellular_automata;
mod cyclic;

pub fn make_boss_arena() -> TileMap {
    let mut out: TileMap = Default::default();
    for x in -BOSS_ARENA_SIZE - 1..=BOSS_ARENA_SIZE + 1 {
        for y in -BOSS_ARENA_SIZE - 1..=BOSS_ARENA_SIZE + 1 {
            out.tiles.insert((x, y), WALL);
        }
    }
    for x in -BOSS_ARENA_SIZE..=BOSS_ARENA_SIZE {
        for y in -BOSS_ARENA_SIZE..=BOSS_ARENA_SIZE {
            out.tiles.insert((x, y), FLOOR);
        }
    }
    out
}
