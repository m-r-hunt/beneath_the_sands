use crate::level_generation::generate_level;
use crate::physics::TileMap;
use crate::player::PlayerControls;
use crate::prelude::*;
use crate::world_generation::Dungeon;
use crate::{CurrentDungeon, Input, ScreenSize, UIState, TILE_SIZE};

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
        ): Self::SystemData,
    ) {
        let offset = screen_size.size / 2.0;
        let mouse_pos = input.raw_mouse_pos - offset;
        for (e, d) in (&entities, &dungeons).join() {
            if input.fire && (d.position - mouse_pos).len2() < 10.0 * 10.0 {
                *ui_state = UIState::Playing;
                let level = generate_level();
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
