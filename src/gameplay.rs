use crate::enemy_ai::{Boss, BossAttack};
use crate::level_generation::{self, BOSS_ARENA_SIZE_Y};
use crate::physics::{
    hitbox_overlap, Bullet, CollidingWithWall, HitBox, PhysicsComponent, TileMap, TILE_SIZE,
};
use crate::player::PlayerControls;
use crate::prelude::*;
use crate::world_map::{CurrentDungeon, Dungeon, Item, Reward};
use crate::{Camera, Event, EventQueue, Input, PlayerProgression, ScreenSize, UIState};

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Team {
    Unaligned,
    Player,
    Enemy,
}

#[derive(Default, Copy, Clone, Eq, PartialEq)]
pub struct TeamWrap {
    pub team: Team,
}

impl Default for Team {
    fn default() -> Self {
        Team::Unaligned
    }
}

impl Component for TeamWrap {
    type Storage = VecStorage<Self>;
}

#[derive(Default)]
pub struct Destructable;

impl Component for Destructable {
    type Storage = HashMapStorage<Self>;
}

#[derive(Default)]
pub struct LevelObject;

impl Component for LevelObject {
    type Storage = VecStorage<Self>;
}

pub struct CollisionHandler;

impl<'a> System<'a> for CollisionHandler {
    type SystemData = (
        Entities<'a>,
        Read<'a, EventQueue>,
        ReadStorage<'a, Destructable>,
    );

    fn run(&mut self, (entities, event_queue, destructables): Self::SystemData) {
        for event in event_queue.iter() {
            if let Event::Collision(entity, _bullet) = event {
                if destructables.get(*entity).is_some() {
                    entities
                        .delete(*entity)
                        .expect("We just got this entity out so it should be valid.");
                }
            }
        }
    }
}

pub struct PenetratingBullet;

impl Component for PenetratingBullet {
    type Storage = VecStorage<Self>;
}

pub struct BulletSelfDestruct;

impl<'a> System<'a> for BulletSelfDestruct {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Bullet>,
        ReadStorage<'a, CollidingWithWall>,
        ReadStorage<'a, PenetratingBullet>,
    );

    fn run(&mut self, (entities, bullets, colliding, penetrating): Self::SystemData) {
        for (entity, _, _, _) in (&entities, &bullets, &colliding, !&penetrating).join() {
            entities
                .delete(entity)
                .expect("We just got this entity out so it should be valid.");
        }
    }
}

#[derive(Default)]
pub struct Exit;

impl Component for Exit {
    type Storage = HashMapStorage<Exit>;
}

pub struct ExitSystem;

// It's possible I should decompose this with an event being fired and handled elsewhere.
// This has a lot of game logic stuffed into basically a collision check with the stairs...
impl<'a> System<'a> for ExitSystem {
    type SystemData = (
        WriteStorage<'a, Transform>,
        ReadStorage<'a, Exit>,
        ReadStorage<'a, PlayerControls>,
        WriteStorage<'a, Camera>,
        Write<'a, UIState>,
        ReadStorage<'a, HitBox>,
        Write<'a, CurrentDungeon>,
        WriteStorage<'a, Dungeon>,
        ReadStorage<'a, LevelObject>,
        Entities<'a>,
        Read<'a, LazyUpdate>,
        Write<'a, TileMap>,
        Read<'a, SimTime>,
        Read<'a, ScreenSize>,
    );

    fn run(
        &mut self,
        (
            mut transforms,
            exits,
            players,
            mut cameras,
            mut ui_state,
            hitboxes,
            current_dungeon,
            mut dungeons,
            level_objects,
            entities,
            lazy_update,
            mut tile_map,
            sim_time,
            screen_size,
        ): Self::SystemData,
    ) {
        let mut exit = false;
        for (exit_transform, exit_hitbox, _) in (&transforms, &hitboxes, &exits).join() {
            for (player_transform, player_hitbox, _) in (&transforms, &hitboxes, &players).join() {
                if hitbox_overlap(player_transform, player_hitbox, exit_transform, exit_hitbox) {
                    exit = true;
                }
            }
        }
        if exit {
            let current_dungeon = current_dungeon
                .entity
                .expect("We should be playing a dungeon when we hit an exit.");
            let current_dungeon = dungeons
                .get_mut(current_dungeon)
                .expect("The current dungeon should be valid when hitting an exit.");
            current_dungeon.completed = true;
            for (_, ent) in (&level_objects, &entities).join() {
                entities.delete(ent).unwrap();
            }
            match current_dungeon.reward {
                Reward::Progress => {
                    // Set up for boss fight
                    lazy_update
                        .create_entity(&entities)
                        .with_boss_prefab()
                        .with(Transform {
                            position: Vector::new(
                                0.0,
                                -(BOSS_ARENA_SIZE_Y as f32 - 2.0) * TILE_SIZE,
                            ),
                        })
                        .with(Boss {
                            attacks: vec![
                                BossAttack::Lines,
                                BossAttack::Sideswipe,
                                BossAttack::RandomBurst,
                            ],
                            attack_cooldown: Timer::new_set(*sim_time, 3.0),
                            ..Default::default()
                        })
                        .build();
                    let dummy_camera_pos = lazy_update
                        .create_entity(&entities)
                        .with_dummy_prefab()
                        .build();
                    for (player_transform, _) in (&mut transforms, &players).join() {
                        player_transform.position = Vector::new(0.0, 100.0);
                    }
                    for (camera_transform, camera) in (&mut transforms, &mut cameras).join() {
                        camera_transform.position = Vector::new(0.0, 0.0) - screen_size.size / 2.0;
                        camera.follow = dummy_camera_pos;
                    }
                    *tile_map = level_generation::make_boss_arena();
                }
                Reward::Choice(_item1, _item2) => {
                    *ui_state = UIState::Choice;
                }
            }
        }
    }
}

const INVINCIBILITY_TIME: f32 = 0.75;

#[derive(Default)]
pub struct Combative {
    pub max_hp: i32,
    pub damage: i32,
    pub invincibility_cooldown: Timer,
}

impl Component for Combative {
    type Storage = VecStorage<Self>;
}

pub struct CombativeCollisionHandler;

impl<'a> System<'a> for CombativeCollisionHandler {
    type SystemData = (
        Entities<'a>,
        Write<'a, EventQueue>,
        WriteStorage<'a, Combative>,
        Read<'a, SimTime>,
        ReadStorage<'a, Bullet>,
    );

    fn run(
        &mut self,
        (entities, mut event_queue, mut combatives, sim_time, bullets): Self::SystemData,
    ) {
        let mut new_events = Vec::new();
        for event in event_queue.iter() {
            if let Event::Collision(entity, bullet_ent) = event {
                if combatives.get(*entity).is_some() {
                    let bullet = bullets.get(*bullet_ent).unwrap();
                    let c = combatives.get_mut(*entity).unwrap();
                    if c.invincibility_cooldown.expired(*sim_time) {
                        c.damage += bullet.damage;
                        if c.damage >= c.max_hp {
                            new_events.push(Event::EntityKilled(*entity));
                        }
                        if !bullet.penetrating {
                            entities.delete(*bullet_ent).unwrap();
                        }
                        c.invincibility_cooldown.set(*sim_time, INVINCIBILITY_TIME);
                    }
                }
            }
        }
        for e in new_events {
            event_queue.enqueue(e);
        }
    }
}

fn apply_upgrade<'a>(
    item: Item,
    players: &mut WriteStorage<'a, PlayerControls>,
    combatives: &mut WriteStorage<'a, Combative>,
    physics: &mut WriteStorage<'a, PhysicsComponent>,
) {
    for p in (players).join() {
        p.items_acquired.push(item);
    }

    match item {
        Item::AttackSpeed => {
            for p in (players).join() {
                p.fire_rate -= 0.1;
            }
        }
        Item::MaxHealth => {
            for (_, c) in (players, combatives).join() {
                c.max_hp += 1;
            }
        }
        Item::TripleShot => {
            for p in (players).join() {
                p.triple_shot = true;
            }
        }
        Item::ExtraDamage => {
            for p in (players).join() {
                p.bullet_damage += 1;
            }
        }
        Item::Penetrating => {
            for p in (players).join() {
                p.penetrating = true;
            }
        }
        Item::ReduceDodgeCooldown => {
            for p in (players).join() {
                p.dodge_cooldown_time = 0.9;
            }
        }
        Item::Backfire => {
            for p in (players).join() {
                p.backfire = true;
            }
        }
        Item::SpeedIncrease => {
            for (physics, _) in (physics, players).join() {
                physics.max_speed = 300.0;
            }
        }
    }
}

pub struct ChoiceSystem;

impl<'a> System<'a> for ChoiceSystem {
    type SystemData = (
        Read<'a, Input>,
        Read<'a, CurrentDungeon>,
        Read<'a, ScreenSize>,
        Write<'a, UIState>,
        WriteStorage<'a, PlayerControls>,
        WriteStorage<'a, Combative>,
        WriteStorage<'a, Dungeon>,
        WriteStorage<'a, PhysicsComponent>,
    );

    fn run(
        &mut self,
        (
            input,
            current_dungeon,
            screen_size,
            mut ui_state,
            mut players,
            mut combatives,
            dungeons,
            mut physics,
        ): Self::SystemData,
    ) {
        let current_dungeon = current_dungeon
            .entity
            .expect("We should be playing a dungeon when we hit are doing choice.");
        let current_dungeon = dungeons
            .get(current_dungeon)
            .expect("The current dungeon should be valid when are doing choice.");
        let mouse_pos = input.raw_mouse_pos;
        if input.clicked {
            if let Reward::Choice(item1, item2) = current_dungeon.reward {
                if mouse_pos.x > screen_size.size.x / 2.0 {
                    apply_upgrade(item2, &mut players, &mut combatives, &mut physics);
                } else {
                    apply_upgrade(item1, &mut players, &mut combatives, &mut physics);
                }
                for (_, c) in (&players, &mut combatives).join() {
                    c.damage = (c.damage - 1).max(0);
                }
                *ui_state = UIState::WorldMap;
            } else {
                panic!("Bad choice state");
            }
        }
    }
}

pub struct BossDeathSystem;

impl<'a> System<'a> for BossDeathSystem {
    type SystemData = (
        Read<'a, EventQueue>,
        ReadStorage<'a, Boss>,
        ReadStorage<'a, LevelObject>,
        Entities<'a>,
        Write<'a, UIState>,
        Write<'a, PlayerProgression>,
        ReadStorage<'a, PlayerControls>,
        WriteStorage<'a, Combative>,
        Write<'a, SoundQueue>,
    );

    fn run(
        &mut self,
        (
            event_queue,
            bosses,
            level_objects,
            entities,
            mut ui_state,
            mut progress,
            players,
            mut combatives,
            mut sound_queue,
        ): Self::SystemData,
    ) {
        for event in event_queue.iter() {
            if let Event::EntityKilled(ent) = event {
                if bosses.get(*ent).is_some() {
                    sound_queue.enqueue(SoundRequest::BossDeath);
                    for (ent, _) in (&entities, &level_objects).join() {
                        entities.delete(ent).unwrap();
                    }
                    if !progress.range_extended {
                        progress.range_extended = true;
                        *ui_state = UIState::WorldMap;
                    } else {
                        *ui_state = UIState::Victory;
                    }
                    for (_, c) in (&players, &mut combatives).join() {
                        c.damage = (c.damage - 1).max(0);
                    }
                }
            }
        }
    }
}

const WAKEUP_RADIUS: f32 = 300.0;

#[derive(Default)]
pub struct Asleep;

impl Component for Asleep {
    type Storage = VecStorage<Self>;
}

pub struct SleepSystem;

impl<'a> System<'a> for SleepSystem {
    type SystemData = (
        WriteStorage<'a, Asleep>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, PlayerControls>,
        Entities<'a>,
        Read<'a, LazyUpdate>,
    );

    fn run(&mut self, (asleeps, transforms, players, entities, lazy_update): Self::SystemData) {
        for (_, player_transform) in (&players, &transforms).join() {
            for (_, sleeper_transform, sleeper) in (&asleeps, &transforms, &entities).join() {
                if (player_transform.position - sleeper_transform.position).len2()
                    <= WAKEUP_RADIUS * WAKEUP_RADIUS
                {
                    lazy_update.remove::<Asleep>(sleeper);
                }
            }
        }
    }
}
