use crate::physics::{Bullet, CollidingWithWall};
use crate::{Event, EventQueue};
use specs::prelude::*;

pub struct CollisionHandler;

impl<'a> System<'a> for CollisionHandler {
    type SystemData = (Entities<'a>, Read<'a, EventQueue>);

    fn run(&mut self, (entities, event_queue): Self::SystemData) {
        for event in event_queue.iter() {
            match event {
                Event::Collision(entity, _bullet) => {
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
