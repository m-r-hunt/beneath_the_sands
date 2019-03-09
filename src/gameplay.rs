use crate::physics::{hitbox_overlap, Bullet, CollidingWithWall, HitBox, TileMap};
use crate::player::PlayerControls;
use crate::prelude::*;
use crate::world_map::{CurrentDungeon, Dungeon, Item, Reward};
use crate::{Event, EventQueue, Input, PlayerProgression, ScreenSize, UIState};

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

pub struct BulletSelfDestruct;

impl<'a> System<'a> for BulletSelfDestruct {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Bullet>,
        ReadStorage<'a, CollidingWithWall>,
    );

    fn run(&mut self, (entities, bullets, colliding): Self::SystemData) {
        for (entity, _, _) in (&entities, &bullets, &colliding).join() {
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
        Write<'a, UIState>,
        ReadStorage<'a, HitBox>,
        Write<'a, CurrentDungeon>,
        WriteStorage<'a, Dungeon>,
        ReadStorage<'a, LevelObject>,
        Entities<'a>,
        Read<'a, LazyUpdate>,
        Write<'a, TileMap>,
    );

    fn run(
        &mut self,
        (
            mut transforms,
            exits,
            players,
            mut ui_state,
            hitboxes,
            current_dungeon,
            mut dungeons,
            level_objects,
            entities,
            lazy_update,
            mut tile_map,
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
                            position: Vector::new(0.0, 0.0),
                        })
                        .build();
                    for (player_transform, _) in (&mut transforms, &players).join() {
                        player_transform.position.x = 0.0;
                        player_transform.position.y = 100.0;
                    }
                    *tile_map = Default::default(); // Todo set up a proper arena
                }
                Reward::Choice(_item1, _item2) => {
                    *ui_state = UIState::Choice;
                }
            }
        }
    }
}

#[derive(Default)]
pub struct Combative {
    pub max_hp: i32,
    pub damage: i32,
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
    );

    fn run(&mut self, (entities, mut event_queue, mut combatives): Self::SystemData) {
        let mut new_events = Vec::new();
        for event in event_queue.iter() {
            if let Event::Collision(entity, bullet) = event {
                if combatives.get(*entity).is_some() {
                    let c = combatives.get_mut(*entity).unwrap();
                    c.damage += 1;
                    if c.damage >= c.max_hp {
                        new_events.push(Event::EntityKilled(*entity));
                    }
                    entities.delete(*bullet).unwrap();
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
    mut players: WriteStorage<'a, PlayerControls>,
    mut combatives: WriteStorage<'a, Combative>,
) {
    match item {
        Item::AttackSpeed => {
            for p in (&mut players).join() {
                p.fire_rate -= 0.1;
            }
        }
        Item::MaxHealth => {
            for (_, c) in (&players, &mut combatives).join() {
                c.max_hp += 1;
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
    );

    fn run(
        &mut self,
        (input, current_dungeon, screen_size, mut ui_state, players, combatives, dungeons): Self::SystemData,
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
                    apply_upgrade(item2, players, combatives);
                } else {
                    apply_upgrade(item1, players, combatives);
                }
                *ui_state = UIState::WorldMap;
            } else {
                panic!("Bad choice state");
            }
        }
    }
}

#[derive(Default)]
pub struct Boss;

impl Component for Boss {
    type Storage = HashMapStorage<Self>;
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
    );

    fn run(
        &mut self,
        (event_queue, bosses, level_objects, entities, mut ui_state, mut progress): Self::SystemData,
    ) {
        for event in event_queue.iter() {
            if let Event::EntityKilled(ent) = event {
                if bosses.get(*ent).is_some() {
                    for (ent, _) in (&entities, &level_objects).join() {
                        entities.delete(ent).unwrap();
                    }
                    if !progress.range_extended {
                        progress.range_extended = true;
                        *ui_state = UIState::WorldMap;
                    } else {
                        *ui_state = UIState::Victory;
                    }
                }
            }
        }
    }
}
