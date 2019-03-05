use crate::level_generation::{generate_level, LevelStyle};
use crate::physics::TileMap;
use crate::player::PlayerControls;
use crate::prelude::*;
use crate::{Input, PlayerProgression, ScreenSize, UIState, TILE_SIZE};

pub const RANGE1: f32 = 150.0;
pub const RANGE2: f32 = 300.0;

pub struct Dungeon {
    pub position: Vector,
    pub reward: bool,
    pub completed: bool,
    pub style: LevelStyle,
}

impl Component for Dungeon {
    type Storage = HashMapStorage<Self>;
}

#[derive(Default)]
pub struct CurrentDungeon {
    pub entity: Option<Entity>,
}

pub struct WorldMapScreen;

impl<'a> System<'a> for WorldMapScreen {
    type SystemData = (
        ReadStorage<'a, Dungeon>,
        Write<'a, UIState>,
        Read<'a, ScreenSize>,
        Read<'a, Input>,
        Write<'a, TileMap>,
        ReadStorage<'a, PlayerControls>,
        WriteStorage<'a, Movement>,
        Read<'a, LazyUpdate>,
        Entities<'a>,
        Write<'a, CurrentDungeon>,
        Read<'a, PlayerProgression>,
    );

    fn run(
        &mut self,
        (
            dungeons,
            mut ui_state,
            screen_size,
            input,
            mut tile_map,
            players,
            mut movements,
            lazy_update,
            entities,
            mut current_dungeon,
            progression,
        ): Self::SystemData,
    ) {
        let offset = screen_size.size / 2.0;
        let mouse_pos = input.raw_mouse_pos - offset;
        for (e, d) in (&entities, &dungeons).join() {
            if input.fire
                && (d.position - mouse_pos).len2() < 10.0 * 10.0
                && (d.position.len2() <= RANGE1 * RANGE1 || progression.range_extended)
                && !d.completed
            {
                *ui_state = UIState::Playing;
                let level = generate_level(d.style);
                *tile_map = level.tile_map;
                for (_, player_movement) in (&players, &mut movements).join() {
                    player_movement.position = Vector::from(level.start_position) * TILE_SIZE;
                }
                lazy_update
                    .create_entity(&entities)
                    .with_exit_prefab()
                    .with(Movement {
                        position: Vector::new(
                            level.exit_position.0 as f32 * TILE_SIZE,
                            level.exit_position.1 as f32 * TILE_SIZE,
                        ),
                        velocity: Vector::new(0.0, 0.0),
                    })
                    .build();
                current_dungeon.entity = Some(e);
            }
        }
    }
}
