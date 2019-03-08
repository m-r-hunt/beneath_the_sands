use crate::physics::TileMap;

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

pub struct GeneratedLevel {
    pub tile_map: TileMap,
    pub start_position: (i32, i32),
    pub exit_position: (i32, i32),
    pub chode_positions: Vec<(i32, i32)>,
}

#[derive(Debug)]
pub struct StringErr(String);

mod cellular_automata;
mod cyclic;
