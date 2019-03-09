use crate::level_generation::{generate_level, LevelStyle};
use crate::physics::TileMap;
use crate::player::PlayerControls;
use crate::prelude::*;
use crate::{Camera, Input, PlayerProgression, ScreenSize, UIState, TILE_SIZE};

pub const RANGE1: f32 = 150.0;
pub const RANGE2: f32 = 300.0;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Item {
    AttackSpeed,         // Done
    MaxHealth,           // Done
    TripleShot,          // Needs work in player probably a flag
    ExtraDamage,         // Extra damage needs work in combative and a var in player
    Penetrating,         // Needs work in combative - probably a bullet flag
    ReduceDodgeCooldown, // Needs work in player var
    SpeedIncrease,       // Needs work in player var
    Backfire,            // Needs work in player probably a flag
}

pub fn all_items() -> Vec<Item> {
    vec![
        Item::AttackSpeed,
        Item::MaxHealth,
        Item::TripleShot,
        Item::ExtraDamage,
        Item::Penetrating,
        Item::ReduceDodgeCooldown,
        Item::SpeedIncrease,
        Item::Backfire,
    ]
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Reward {
    Choice(Item, Item),
    Progress,
}

pub struct Dungeon {
    pub position: Vector,
    pub reward: Reward,
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
        WriteStorage<'a, Transform>,
        Read<'a, LazyUpdate>,
        Entities<'a>,
        Write<'a, CurrentDungeon>,
        Read<'a, PlayerProgression>,
        WriteStorage<'a, Camera>,
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
            mut transforms,
            lazy_update,
            entities,
            mut current_dungeon,
            progression,
            mut cameras,
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
                let mut player_start_position = Vector::new(-1.0, -1.0);
                for (_, player_movement) in (&players, &mut transforms).join() {
                    player_movement.position = Vector::from(level.start_position) * TILE_SIZE
                        + Vector::new(TILE_SIZE / 2.0, TILE_SIZE / 2.0);
                    player_start_position = player_movement.position;
                }
                lazy_update
                    .create_entity(&entities)
                    .with_exit_prefab()
                    .with(Transform {
                        position: Vector::new(
                            level.exit_position.0 as f32 * TILE_SIZE,
                            level.exit_position.1 as f32 * TILE_SIZE,
                        ),
                    })
                    .build();
                for cp in level.chode_positions {
                    lazy_update
                        .create_entity(&entities)
                        .with_chode_prefab()
                        .with(Transform {
                            position: Vector::new(
                                cp.0 as f32 * TILE_SIZE + TILE_SIZE / 2.0,
                                cp.1 as f32 * TILE_SIZE + TILE_SIZE / 2.0,
                            ),
                        })
                        .build();
                }
                current_dungeon.entity = Some(e);
                for (camera, transform) in (&mut cameras, &mut transforms).join() {
                    for (ent, _) in (&entities, &players).join() {
                        camera.follow = ent;
                    }
                    transform.position = player_start_position - screen_size.size / 2.0;
                }
            }
        }
    }
}
