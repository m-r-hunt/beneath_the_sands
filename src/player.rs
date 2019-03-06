use crate::physics::PhysicsComponent;
use crate::prelude::*;
use crate::{Input, SimTime, Timer};

const PLAYER_ACCELERATION: f32 = 10.0;

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
        WriteStorage<'a, Transform>,
        WriteStorage<'a, PhysicsComponent>,
        Read<'a, Input>,
        Read<'a, SimTime>,
        Read<'a, LazyUpdate>,
        Entities<'a>,
    );

    fn run(
        &mut self,
        (mut player_controls, mut transforms, mut physics, input, sim_time, lazy_update, entities): Self::SystemData,
    ) {
        for (player_controls, transform, physics) in
            (&mut player_controls, &mut transforms, &mut physics).join()
        {
            physics.acceleration = Vector::new(0.0, 0.0);
            if input.left {
                physics.acceleration.x = -1.0;
            }
            if input.right {
                physics.acceleration.x = 1.0;
            }
            if input.up {
                physics.acceleration.y = -1.0;
            }
            if input.down {
                physics.acceleration.y = 1.0;
            }
            if physics.acceleration.len2() >= std::f32::EPSILON {
                physics.acceleration = physics.acceleration.with_len(PLAYER_ACCELERATION);
            } else if physics.velocity.len2() >= std::f32::EPSILON {
                physics.velocity = physics
                    .velocity
                    .with_len((physics.velocity.len() - 5.0).max(0.0));
            }
            if input.fire && player_controls.fire_cooldown.expired(*sim_time) {
                let bullet_speed = 10.0;
                let velocity = (input.mouse_pos - transform.position).with_len(bullet_speed);
                let position = transform.position + velocity.with_len(30.0);
                lazy_update
                    .create_entity(&entities)
                    .with_bullet_prefab()
                    .with(Transform { position })
                    .with(PhysicsComponent {
                        velocity,
                        max_speed: bullet_speed,
                        ..Default::default()
                    })
                    .build();
                player_controls.fire_cooldown.set(*sim_time, 0.7);
            }
        }
    }
}
