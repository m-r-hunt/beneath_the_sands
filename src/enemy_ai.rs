use crate::prelude::*;
use crate::{Event, EventQueue};

#[derive(Default)]
pub struct ChodeAI;

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
