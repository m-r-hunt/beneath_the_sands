use crate::gameplay::{Team, TeamWrap};
use crate::physics::{check_collision, Bullet, HitBox, PhysicsComponent, TileMap};
use crate::prelude::*;
use crate::sound::{SoundQueue, SoundRequest};
use crate::world_map::Item;
use crate::{Event, EventQueue, UIState};
use crate::{Input, SimTime, Timer};

const PLAYER_ACCELERATION: f32 = 1000.0;
const DODGE_DISTANCE: i32 = 45;

#[derive(Default)]
pub struct PlayerControls {
    pub fire_rate: f32,
    pub fire_cooldown: Timer,
    pub dodge_cooldown: Timer,
    pub triple_shot: bool,
    pub bullet_damage: i32,
    pub penetrating: bool,
    pub dodge_cooldown_time: f32,
    pub backfire: bool,
    pub items_acquired: Vec<Item>,
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
        ReadStorage<'a, HitBox>,
        Read<'a, Input>,
        Read<'a, SimTime>,
        Read<'a, LazyUpdate>,
        Read<'a, TileMap>,
        Entities<'a>,
        Write<'a, SoundQueue>,
    );

    fn run(
        &mut self,
        (
            mut player_controls,
            mut transforms,
            mut physics,
            hitboxes,
            input,
            sim_time,
            lazy_update,
            tile_map,
            entities,
            mut sound_queue,
        ): Self::SystemData,
    ) {
        for (player_controls, transform, physics, player_ent) in (
            &mut player_controls,
            &mut transforms,
            &mut physics,
            &entities,
        )
            .join()
        {
            physics.acceleration = Vector::new(0.0, 0.0);
            let mut dx = 0;
            let mut dy = 0;
            if input.left {
                physics.acceleration.x = -1.0;
                dx = -1;
            }
            if input.right {
                physics.acceleration.x = 1.0;
                dx = 1;
            }
            if input.up {
                physics.acceleration.y = -1.0;
                dy = -1;
            }
            if input.down {
                physics.acceleration.y = 1.0;
                dy = 1;
            }
            if physics.acceleration.len2() >= std::f32::EPSILON {
                physics.acceleration = physics.acceleration.with_len(PLAYER_ACCELERATION);
            } else if physics.velocity.len2() >= std::f32::EPSILON {
                physics.velocity = physics
                    .velocity
                    .with_len((physics.velocity.len() - 90.0).max(0.0));
            }
            if input.fire && player_controls.fire_cooldown.expired(*sim_time) {
                let bullet_speed = 400.0;
                let velocity = (input.mouse_pos - transform.position).with_len(bullet_speed);
                let position = transform.position + velocity.with_len(30.0);
                lazy_update
                    .create_entity(&entities)
                    .with_bullet_prefab()
                    .with(Bullet {
                        radius: 5.0,
                        damage: player_controls.bullet_damage,
                        penetrating: player_controls.penetrating,
                    })
                    .with(Transform { position })
                    .with(PhysicsComponent {
                        velocity,
                        max_speed: bullet_speed,
                        ..Default::default()
                    })
                    .with(TeamWrap { team: Team::Player })
                    .build();
                if player_controls.triple_shot {
                    let velocity =
                        Vector::from_angle((input.mouse_pos - transform.position).angle() + 20.0)
                            .with_len(bullet_speed);
                    let position = transform.position + velocity.with_len(30.0);
                    lazy_update
                        .create_entity(&entities)
                        .with_bullet_prefab()
                        .with(Bullet {
                            radius: 5.0,
                            damage: player_controls.bullet_damage,
                            penetrating: player_controls.penetrating,
                        })
                        .with(Transform { position })
                        .with(PhysicsComponent {
                            velocity,
                            max_speed: bullet_speed,
                            ..Default::default()
                        })
                        .with(TeamWrap { team: Team::Player })
                        .build();

                    let velocity =
                        Vector::from_angle((input.mouse_pos - transform.position).angle() - 20.0)
                            .with_len(bullet_speed);
                    let position = transform.position + velocity.with_len(30.0);
                    lazy_update
                        .create_entity(&entities)
                        .with_bullet_prefab()
                        .with(Bullet {
                            radius: 5.0,
                            damage: player_controls.bullet_damage,
                            penetrating: player_controls.penetrating,
                        })
                        .with(Transform { position })
                        .with(PhysicsComponent {
                            velocity,
                            max_speed: bullet_speed,
                            ..Default::default()
                        })
                        .with(TeamWrap { team: Team::Player })
                        .build();
                }
                if player_controls.backfire {
                    let velocity = (transform.position - input.mouse_pos).with_len(bullet_speed);
                    let position = transform.position + velocity.with_len(30.0);
                    lazy_update
                        .create_entity(&entities)
                        .with_bullet_prefab()
                        .with(Bullet {
                            radius: 5.0,
                            damage: player_controls.bullet_damage,
                            penetrating: player_controls.penetrating,
                        })
                        .with(Transform { position })
                        .with(PhysicsComponent {
                            velocity,
                            max_speed: bullet_speed,
                            ..Default::default()
                        })
                        .with(TeamWrap { team: Team::Player })
                        .build();
                }
                sound_queue.enqueue(SoundRequest::PlayerShot);
                player_controls
                    .fire_cooldown
                    .set(*sim_time, player_controls.fire_rate);
            }
            if input.dodge
                && player_controls.dodge_cooldown.expired(*sim_time)
                && (dx != 0 || dy != 0)
                && hitboxes.get(player_ent).is_some()
            {
                player_controls
                    .dodge_cooldown
                    .set(*sim_time, player_controls.dodge_cooldown_time);
                let hitbox = hitboxes.get(player_ent).unwrap();
                let mut the_position = (
                    transform.position.x.floor() as i32,
                    transform.position.y.floor() as i32,
                );
                physics.velocity = Vector::new(0.0, 0.0);
                for _i in 0..DODGE_DISTANCE {
                    let new_position = (the_position.0 + dx, the_position.1 + dy);
                    let colliding = check_collision(
                        Vector::new(new_position.0 as f32, new_position.1 as f32),
                        hitbox,
                        &tile_map,
                    );
                    if !colliding {
                        the_position = new_position
                    } else {
                        break;
                    }
                }
                transform.position = Vector::new(the_position.0 as f32, the_position.1 as f32);
            }
        }
    }
}

pub struct PlayerDeath;

impl<'a> System<'a> for PlayerDeath {
    type SystemData = (
        Read<'a, EventQueue>,
        ReadStorage<'a, PlayerControls>,
        Write<'a, UIState>,
        Write<'a, SoundQueue>,
    );

    fn run(&mut self, (event_queue, players, mut ui_state, mut sound_queue): Self::SystemData) {
        for event in event_queue.iter() {
            if let Event::EntityKilled(ent) = event {
                if players.get(*ent).is_some() {
                    *ui_state = UIState::GameOver;
                    sound_queue.enqueue(SoundRequest::PlayerDeath);
                }
            }
        }
    }
}
