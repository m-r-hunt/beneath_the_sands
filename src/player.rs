use super::physics::{Bullet, Movement};
use super::render::RenderComponent;
use super::{Input, SimTime, Timer, WorldBounds};
use quicksilver::graphics::Color;
use specs::prelude::*;

const PLAYER_SPEED: f32 = 2.0;

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
            movement.velocity = (0.0, 0.0);
            if input.left {
                movement.velocity.0 = -1.0;
            }
            if input.right {
                movement.velocity.0 = 1.0;
            }
            if input.up {
                movement.velocity.1 = -1.0;
            }
            if input.down {
                movement.velocity.1 = 1.0;
            }
            let vel_len = (movement.velocity.0 * movement.velocity.0 + movement.velocity.0 * movement.velocity.0).sqrt();
            if vel_len > std::f32::EPSILON {
                movement.velocity.0 /= vel_len;
                movement.velocity.1 /= vel_len;
            }
            movement.velocity.0 *= PLAYER_SPEED;
            movement.velocity.1 *= PLAYER_SPEED;
            if input.fire && player_controls.fire_cooldown.expired(*sim_time) {
                lazy_update
                    .create_entity(&entities)
                    .with(Movement {
                        position: (movement.position.0, movement.position.1 - 30.0),
                        velocity: (0.0, -10.0),
                    })
                    .with(RenderComponent {
                        width: 5.0,
                        height: 5.0,
                        colour: Color::YELLOW,
                    })
                    .with(Bullet)
                    .build();
                player_controls.fire_cooldown.set(*sim_time, 1.0 / 10.0);
            }
        }
    }
}

pub struct SoftBoundsCheck;

impl<'a> System<'a> for SoftBoundsCheck {
    type SystemData = (
        WriteStorage<'a, Movement>,
        ReadStorage<'a, PlayerControls>,
        Read<'a, WorldBounds>,
    );

    fn run(&mut self, (mut movement, players, bounds): Self::SystemData) {
        for (movement, _) in (&mut movement, &players).join() {
            movement.position.0 = movement.position.0.max(bounds.left);
            movement.position.0 = movement.position.0.min(bounds.right);
            movement.position.1 = movement.position.1.max(bounds.top);
            movement.position.1 = movement.position.1.min(bounds.bottom);
        }
    }
}
