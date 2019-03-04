use crate::physics::{hitbox_overlap, Bullet, CollidingWithWall, HitBox};
use crate::player::PlayerControls;
use crate::prelude::*;
use crate::world_generation::Dungeon;
use crate::UIState;
use crate::{CurrentDungeon, Event, EventQueue};

#[derive(Default)]
pub struct Destructable;

impl Component for Destructable {
    type Storage = HashMapStorage<Self>;
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
            match event {
                Event::Collision(entity, _bullet) => {
                    if destructables.get(*entity).is_some() {
                        entities
                            .delete(*entity)
                            .expect("We just got this entity out so it should be valid.");
                    }
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

impl<'a> System<'a> for ExitSystem {
    type SystemData = (
        ReadStorage<'a, Movement>,
        ReadStorage<'a, Exit>,
        ReadStorage<'a, PlayerControls>,
        Write<'a, UIState>,
        ReadStorage<'a, HitBox>,
        Write<'a, CurrentDungeon>,
        WriteStorage<'a, Dungeon>,
    );

    fn run(
        &mut self,
        (movements, exits, players, mut ui_state, hitboxes, current_dungeon, mut dungeons): Self::SystemData,
    ) {
        for (exit_movement, exit_hitbox, _) in (&movements, &hitboxes, &exits).join() {
            for (player_movement, player_hitbox, _) in (&movements, &hitboxes, &players).join() {
                if hitbox_overlap(player_movement, player_hitbox, exit_movement, exit_hitbox) {
                    *ui_state = UIState::WorldMap;
                    let current_dungeon = current_dungeon
                        .entity
                        .expect("We should be playing a dungeon when we hit an exit.");
                    dungeons
                        .get_mut(current_dungeon)
                        .expect("The current dungeon should be valid when hitting an exit.")
                        .completed = true;
                }
            }
        }
    }
}
