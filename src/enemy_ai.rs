use crate::gameplay::{Team, TeamWrap};
use crate::physics::PhysicsComponent;
use crate::player::PlayerControls;
use crate::prelude::*;
use crate::{Event, EventQueue};

const TARGET_DISTANCE: f32 = 100.0;
const CHODE_ACCELERATION: f32 = 300.0;
const CHODE_FIRE_COOLDOWN: f32 = 1.0;

#[derive(Default)]
pub struct ChodeAI {
    pub fire_cooldown: Timer,
}

impl Component for ChodeAI {
    type Storage = HashMapStorage<Self>;
}

pub struct ChodeDeath;

impl<'a> System<'a> for ChodeDeath {
    type SystemData = (Entities<'a>, Read<'a, EventQueue>, ReadStorage<'a, ChodeAI>);

    fn run(&mut self, (entities, event_queue, chode_ais): Self::SystemData) {
        for event in event_queue.iter() {
            if let Event::EntityKilled(ent) = event {
                if chode_ais.get(*ent).is_some() {
                    entities.delete(*ent).unwrap();
                }
            }
        }
    }
}

pub struct RunChodeAI;

impl<'a> System<'a> for RunChodeAI {
    type SystemData = (
        WriteStorage<'a, ChodeAI>,
        WriteStorage<'a, PhysicsComponent>,
        ReadStorage<'a, PlayerControls>,
        ReadStorage<'a, Transform>,
        Read<'a, SimTime>,
        Entities<'a>,
        Read<'a, LazyUpdate>,
    );

    fn run(
        &mut self,
        (mut chode_ais, mut physics, player_controls, transforms, sim_time, entities, lazy_update): Self::SystemData,
    ) {
        let mut player_pos = Vector::new(0.0, 0.0);
        for (_, player_transform) in (&player_controls, &transforms).join() {
            player_pos = player_transform.position;
        }

        for (chode, transform, physics) in (&mut chode_ais, &transforms, &mut physics).join() {
            let target_point =
                player_pos + (transform.position - player_pos).with_len(TARGET_DISTANCE);
            let dir = target_point - transform.position;
            if dir.len2() >= std::f32::EPSILON {
                physics.acceleration = dir.with_len(CHODE_ACCELERATION);
            }
            if dir.len2() < 50.0 * 50.0 {
                physics.acceleration *= dir.len2() / 50.0 * 50.0;
                physics.velocity *= dir.len2() / 50.0 * 50.0;
            }

            if chode.fire_cooldown.expired(*sim_time) {
                let bullet_speed = 400.0;
                let velocity = (player_pos - transform.position).with_len(bullet_speed);
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
                    .with(TeamWrap { team: Team::Enemy })
                    .build();
                chode.fire_cooldown.set(*sim_time, CHODE_FIRE_COOLDOWN);
            }
        }
    }
}
