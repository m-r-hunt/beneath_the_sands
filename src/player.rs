use super::physics::Movement;
use super::{Input, SimTime, Timer};
use crate::prefabs::PrefabBuilder;
use quicksilver::geom::Vector;
use specs::prelude::*;

const PLAYER_SPEED: f32 = 5.0;

#[derive(Default)]
pub struct PlayerControls {
    pub fire_cooldown: Timer,
}

impl Component for PlayerControls {
    type Storage = HashMapStorage<Self>;
}

pub struct PlayerControlSystem;

impl<'a> System<'a> for PlayerControlSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteStorage<'a, PlayerControls>,
        WriteStorage<'a, Movement>,
        Read<'a, Input>,
        Read<'a, SimTime>,
        Read<'a, LazyUpdate>,
        Entities<'a>,
    );

    fn run(
        &mut self,
        (mut player_controls, mut movements, input, sim_time, lazy_update, entities): Self::SystemData,
    ) {
        for (player_controls, movement) in (&mut player_controls, &mut movements).join() {
            movement.velocity = Vector::new(0.0, 0.0);
            if input.left {
                movement.velocity.x = -1.0;
            }
            if input.right {
                movement.velocity.x = 1.0;
            }
            if input.up {
                movement.velocity.y = -1.0;
            }
            if input.down {
                movement.velocity.y = 1.0;
            }
            let vel_len = (movement.velocity.x * movement.velocity.x
                + movement.velocity.x * movement.velocity.x)
                .sqrt();
            if vel_len > std::f32::EPSILON {
                movement.velocity.x /= vel_len;
                movement.velocity.y /= vel_len;
            }
            movement.velocity.x *= PLAYER_SPEED;
            movement.velocity.y *= PLAYER_SPEED;
            if input.fire && player_controls.fire_cooldown.expired(*sim_time) {
                let velocity = (input.mouse_pos - movement.position).with_len(10.0);
                let position = movement.position + velocity.with_len(30.0);
                lazy_update
                    .create_entity(&entities)
                    .with_bullet_prefab()
                    .with(Movement { position, velocity })
                    .build();
                player_controls.fire_cooldown.set(*sim_time, 1.0 / 10.0);
            }
        }
    }
}
