use crate::physics::PhysicsComponent;
use crate::prelude::*;
use crate::{Input, SimTime, Timer};

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
            physics.velocity = Vector::new(0.0, 0.0);
            if input.left {
                physics.velocity.x = -1.0;
            }
            if input.right {
                physics.velocity.x = 1.0;
            }
            if input.up {
                physics.velocity.y = -1.0;
            }
            if input.down {
                physics.velocity.y = 1.0;
            }
            let vel_len = (physics.velocity.x * physics.velocity.x
                + physics.velocity.x * physics.velocity.x)
                .sqrt();
            if vel_len > std::f32::EPSILON {
                physics.velocity.x /= vel_len;
                physics.velocity.y /= vel_len;
            }
            physics.velocity.x *= PLAYER_SPEED;
            physics.velocity.y *= PLAYER_SPEED;
            if input.fire && player_controls.fire_cooldown.expired(*sim_time) {
                let velocity = (input.mouse_pos - transform.position).with_len(10.0);
                let position = transform.position + velocity.with_len(30.0);
                lazy_update
                    .create_entity(&entities)
                    .with_bullet_prefab()
                    .with(Transform { position })
                    .with(PhysicsComponent {
                        velocity,
                        ..Default::default()
                    })
                    .build();
                player_controls.fire_cooldown.set(*sim_time, 1.0 / 10.0);
            }
        }
    }
}
