use crate::gameplay::{Asleep, PenetratingBullet, Team, TeamWrap};
use crate::physics::PhysicsComponent;
use crate::player::PlayerControls;
use crate::prelude::*;
use crate::{Event, EventQueue};

const TARGET_DISTANCE: f32 = 100.0;
const CHODE_ACCELERATION: f32 = 300.0;
const CHODE_FIRE_COOLDOWN: f32 = 1.0;
const SHOTGUNNER_ACCELERATION: f32 = 200.0;
const SHOTGUNNER_FIRE_COOLDOWN: f32 = 2.0;
const SPINNER_ACCELERATION: f32 = 300.0;
const SPINNER_FIRE_COOLDOWN: f32 = 1.0;

#[derive(Default)]
pub struct ChodeAI {
    pub fire_cooldown: Timer,
}

impl Component for ChodeAI {
    type Storage = HashMapStorage<Self>;
}

#[derive(Default)]
pub struct ShotgunnerAI {
    pub fire_cooldown: Timer,
}

impl Component for ShotgunnerAI {
    type Storage = HashMapStorage<Self>;
}

#[derive(Default)]
pub struct SpinnerAI {
    pub fire_cooldown: Timer,
}

impl Component for SpinnerAI {
    type Storage = HashMapStorage<Self>;
}

pub struct ChodeDeath;

impl<'a> System<'a> for ChodeDeath {
    type SystemData = (
        Entities<'a>,
        Read<'a, EventQueue>,
        ReadStorage<'a, ChodeAI>,
        ReadStorage<'a, ShotgunnerAI>,
        ReadStorage<'a, SpinnerAI>,
    );

    fn run(
        &mut self,
        (entities, event_queue, chode_ais, shotgunner_ais, spinner_ais): Self::SystemData,
    ) {
        for event in event_queue.iter() {
            if let Event::EntityKilled(ent) = event {
                if chode_ais.get(*ent).is_some()
                    || shotgunner_ais.get(*ent).is_some()
                    || spinner_ais.get(*ent).is_some()
                {
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
        ReadStorage<'a, Asleep>,
        Read<'a, SimTime>,
        Entities<'a>,
        Read<'a, LazyUpdate>,
    );

    fn run(
        &mut self,
        (
            mut chode_ais,
            mut physics,
            player_controls,
            transforms,
            asleeps,
            sim_time,
            entities,
            lazy_update,
        ): Self::SystemData,
    ) {
        let mut player_pos = Vector::new(0.0, 0.0);
        for (_, player_transform) in (&player_controls, &transforms).join() {
            player_pos = player_transform.position;
        }

        for (chode, transform, physics, _) in
            (&mut chode_ais, &transforms, &mut physics, !&asleeps).join()
        {
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

pub struct RunShotgunnerAI;

impl<'a> System<'a> for RunShotgunnerAI {
    type SystemData = (
        WriteStorage<'a, ShotgunnerAI>,
        WriteStorage<'a, PhysicsComponent>,
        ReadStorage<'a, PlayerControls>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, Asleep>,
        Read<'a, SimTime>,
        Entities<'a>,
        Read<'a, LazyUpdate>,
    );

    fn run(
        &mut self,
        (
            mut shotgunner_ais,
            mut physics,
            player_controls,
            transforms,
            asleeps,
            sim_time,
            entities,
            lazy_update,
        ): Self::SystemData,
    ) {
        let mut player_pos = Vector::new(0.0, 0.0);
        for (_, player_transform) in (&player_controls, &transforms).join() {
            player_pos = player_transform.position;
        }

        for (shotgunner, transform, physics, _) in
            (&mut shotgunner_ais, &transforms, &mut physics, !&asleeps).join()
        {
            let target_point =
                player_pos + (transform.position - player_pos).with_len(TARGET_DISTANCE);
            let dir = target_point - transform.position;
            if dir.len2() >= std::f32::EPSILON {
                physics.acceleration = dir.with_len(SHOTGUNNER_ACCELERATION);
            }
            if dir.len2() < 50.0 * 50.0 {
                physics.acceleration *= dir.len2() / 50.0 * 50.0;
                physics.velocity *= dir.len2() / 50.0 * 50.0;
            }

            if shotgunner.fire_cooldown.expired(*sim_time) {
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
                for bullet in 1..3 {
                    let velocity = Vector::from_angle(
                        (player_pos - transform.position).angle() + 10.0 * bullet as f32,
                    )
                    .with_len(bullet_speed);
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
                    let velocity = Vector::from_angle(
                        (player_pos - transform.position).angle() - 10.0 * bullet as f32,
                    )
                    .with_len(bullet_speed);
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
                }
                shotgunner
                    .fire_cooldown
                    .set(*sim_time, SHOTGUNNER_FIRE_COOLDOWN);
            }
        }
    }
}

pub struct RunSpinnerAI;

impl<'a> System<'a> for RunSpinnerAI {
    type SystemData = (
        WriteStorage<'a, SpinnerAI>,
        WriteStorage<'a, PhysicsComponent>,
        ReadStorage<'a, PlayerControls>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, Asleep>,
        Read<'a, SimTime>,
        Entities<'a>,
        Read<'a, LazyUpdate>,
    );

    fn run(
        &mut self,
        (
            mut spinner_ais,
            mut physics,
            player_controls,
            transforms,
            asleeps,
            sim_time,
            entities,
            lazy_update,
        ): Self::SystemData,
    ) {
        let mut player_pos = Vector::new(0.0, 0.0);
        for (_, player_transform) in (&player_controls, &transforms).join() {
            player_pos = player_transform.position;
        }

        for (spinner, transform, physics, _) in
            (&mut spinner_ais, &transforms, &mut physics, !&asleeps).join()
        {
            let target_point =
                player_pos + (transform.position - player_pos).with_len(TARGET_DISTANCE);
            let dir = target_point - transform.position;
            if dir.len2() >= std::f32::EPSILON {
                physics.acceleration = dir.with_len(SPINNER_ACCELERATION);
            }
            physics.acceleration +=
                Vector::from_angle((transform.position - player_pos).angle() + 90.0)
                    .with_len(SPINNER_ACCELERATION);
            physics.acceleration = physics.acceleration.with_len(SPINNER_ACCELERATION);

            if spinner.fire_cooldown.expired(*sim_time) {
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
                spinner.fire_cooldown.set(*sim_time, SPINNER_FIRE_COOLDOWN);
            }
        }
    }
}

#[derive(Default)]
pub struct Boss {
    pub attacks: Vec<BossAttack>,
    pub current_attack: usize,
    pub attack_cooldown: Timer,
}

impl Component for Boss {
    type Storage = HashMapStorage<Self>;
}

pub enum BossAttack {
    Lines,
    Sideswipe,
}

pub struct RunBossAI;

impl<'a> System<'a> for RunBossAI {
    type SystemData = (
        ReadStorage<'a, Transform>,
        WriteStorage<'a, Boss>,
        Entities<'a>,
        Read<'a, LazyUpdate>,
        Read<'a, SimTime>,
    );

    fn run(&mut self, (transforms, mut bosses, entities, lazy_update, sim_time): Self::SystemData) {
        for (transform, boss) in (&transforms, &mut bosses).join() {
            if boss.attack_cooldown.expired(*sim_time) {
                match boss.attacks[boss.current_attack] {
                    BossAttack::Lines => {
                        for line in 0..3 {
                            let angle1 = line as f32 * 10.0;
                            let angle2 = -angle1;
                            for bullet in 0..10 {
                                let position = transform.position
                                    + Vector::from_angle(90.0 + angle1).with_len(70.0);
                                let speed = 100.0 * (bullet as f32 + 1.0);
                                let velocity = Vector::from_angle(90.0 + angle1).with_len(speed);
                                lazy_update
                                    .create_entity(&entities)
                                    .with_bullet_prefab()
                                    .with(Transform { position })
                                    .with(PhysicsComponent {
                                        velocity,
                                        max_speed: speed,
                                        ..Default::default()
                                    })
                                    .with(TeamWrap { team: Team::Enemy })
                                    .with(PenetratingBullet)
                                    .build();
                                let position = transform.position
                                    + Vector::from_angle(90.0 + angle2).with_len(70.0);
                                let speed = 100.0 * (bullet as f32 + 1.0);
                                let velocity = Vector::from_angle(90.0 + angle2).with_len(speed);
                                lazy_update
                                    .create_entity(&entities)
                                    .with_bullet_prefab()
                                    .with(Transform { position })
                                    .with(PhysicsComponent {
                                        velocity,
                                        max_speed: speed,
                                        ..Default::default()
                                    })
                                    .with(TeamWrap { team: Team::Enemy })
                                    .with(PenetratingBullet)
                                    .build();
                            }
                        }
                    }
                    BossAttack::Sideswipe => {
                        let speed = 100.0;
                        for line in 0..6 {
                            let y = 100.0 * line as f32;
                            for bullet in 0..6 {
                                let position = Vector::new(-500.0 + 20.0 * bullet as f32, y);
                                lazy_update
                                    .create_entity(&entities)
                                    .with_bullet_prefab()
                                    .with(Transform { position })
                                    .with(PhysicsComponent {
                                        velocity: Vector::new(speed, 0.0),
                                        max_speed: speed,
                                        ..Default::default()
                                    })
                                    .with(TeamWrap { team: Team::Enemy })
                                    .with(PenetratingBullet)
                                    .build();
                                let y = y + 50.0;
                                let position = Vector::new(500.0 - 20.0 * bullet as f32, y);
                                lazy_update
                                    .create_entity(&entities)
                                    .with_bullet_prefab()
                                    .with(Transform { position })
                                    .with(PhysicsComponent {
                                        velocity: Vector::new(-speed, 0.0),
                                        max_speed: speed,
                                        ..Default::default()
                                    })
                                    .with(TeamWrap { team: Team::Enemy })
                                    .with(PenetratingBullet)
                                    .build();
                            }
                        }
                    }
                }
                boss.attack_cooldown.set(*sim_time, 5.0);
                boss.current_attack = (boss.current_attack + 1) % boss.attacks.len();
            }
        }
    }
}
