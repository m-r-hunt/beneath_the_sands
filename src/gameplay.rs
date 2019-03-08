use crate::physics::{hitbox_overlap, Bullet, CollidingWithWall, HitBox};
use crate::player::PlayerControls;
use crate::prelude::*;
use crate::world_map::{CurrentDungeon, Dungeon, Reward};
use crate::UIState;
use crate::{Event, EventQueue};

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
        ReadStorage<'a, Transform>,
        ReadStorage<'a, Exit>,
        ReadStorage<'a, PlayerControls>,
        Write<'a, UIState>,
        ReadStorage<'a, HitBox>,
        Write<'a, CurrentDungeon>,
        WriteStorage<'a, Dungeon>,
        ReadStorage<'a, LevelObject>,
        Entities<'a>,
    );

    fn run(
        &mut self,
        (
            transforms,
            exits,
            players,
            mut ui_state,
            hitboxes,
            current_dungeon,
            mut dungeons,
            level_objects,
            entities,
        ): Self::SystemData,
    ) {
        for (exit_transform, exit_hitbox, _) in (&transforms, &hitboxes, &exits).join() {
            for (player_transform, player_hitbox, _) in (&transforms, &hitboxes, &players).join() {
                if hitbox_overlap(player_transform, player_hitbox, exit_transform, exit_hitbox) {
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
                            *ui_state = UIState::BossFight;
                        }
                        Reward::Choice(_item1, _item2) => {
                            *ui_state = UIState::Choice;
                        }
                    }
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
