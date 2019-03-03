use crate::{Event, EventQueue};
use specs::prelude::*;

pub struct CollisionHandler;

impl<'a> System<'a> for CollisionHandler {
    type SystemData = (Entities<'a>, Read<'a, EventQueue>);

    fn run(&mut self, (entities, event_queue): Self::SystemData) {
        for event in event_queue.iter() {
            match event {
                Event::Collision(entity, _bullet) => {
                    entities.delete(*entity).unwrap();
                }
            }
        }
    }
}
