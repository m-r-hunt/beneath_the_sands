use super::physics::{HitBox, Movement};
use super::render::RenderComponent;
use super::{Event, EventQueue, SimTime, Timer, WorldBounds};
use quicksilver::graphics::Color;
use specs::prelude::*;

pub struct HardBoundsCheck {
    pub padding: f32,
}

impl<'a> System<'a> for HardBoundsCheck {
    type SystemData = (
        ReadStorage<'a, Movement>,
        Entities<'a>,
        Read<'a, WorldBounds>,
    );

    fn run(&mut self, (movement, entities, bounds): Self::SystemData) {
        for (movement, entity) in (&movement, &*entities).join() {
            if movement.position.0 < bounds.left - self.padding
                || movement.position.0 > bounds.right + self.padding
                || movement.position.1 < bounds.top - self.padding
                || movement.position.1 > bounds.bottom + self.padding
            {
                entities.delete(entity).unwrap();
            }
        }
    }
}

pub struct CollisionHandler;

impl<'a> System<'a> for CollisionHandler {
    type SystemData = (Read<'a, EventQueue>, Entities<'a>);

    fn run(&mut self, (event_queue, entities): Self::SystemData) {
        for event in event_queue.iter() {
            match event {
                Event::Collision(hit, hit_by) => {
                    entities.delete(*hit).unwrap();
                    entities.delete(*hit_by).unwrap();
                }
            }
        }
    }
}

#[derive(Default)]
pub struct Spawned;

impl Component for Spawned {
    type Storage = NullStorage<Self>;
}

pub struct Spawner {
    pub waves: Vec<Wave>,
    pub current_wave: usize,
    pub current_repeat: i32,
    pub repeat_cooldown: Timer,
}

pub struct Wave {
    pub spawn_fn: Box<Fn(&LazyUpdate, &Entities) + Send>,
    pub repeats: i32,
    pub delay: f32,
}

impl<'a> System<'a> for Spawner {
    type SystemData = (
        Read<'a, LazyUpdate>,
        Entities<'a>,
        ReadStorage<'a, Spawned>,
        Read<'a, SimTime>,
    );

    fn run(&mut self, (lazy_update, entities, spawned, sim_time): Self::SystemData) {
        let number_remaining = spawned.join().count();
        if self.repeat_cooldown.expired(*sim_time) {
            if number_remaining == 0 && self.current_wave < self.waves.len() && self.current_repeat >= self.waves[self.current_wave].repeats  {
                self.current_wave += 1;
                self.current_repeat = 0;
            }
            if self.current_wave < self.waves.len() {
                let current_wave = &self.waves[self.current_wave];
                if self.current_repeat < current_wave.repeats {
                    (current_wave.spawn_fn)(&lazy_update, &entities);
                    self.current_repeat += 1;
                    self.repeat_cooldown.set(*sim_time, current_wave.delay);
                }
            }
        }
    }
}

pub fn spawn_chode(coords: (f32, f32), velocity: (f32, f32), lazy_update: &LazyUpdate, entities: &Entities) {
    lazy_update
        .create_entity(entities)
        .with(Movement {
            position: coords,
            velocity: velocity,
        })
        .with(Spawned)
        .with(RenderComponent {
            width: 20.0,
            height: 20.0,
            colour: Color::RED,
        })
        .with(HitBox {
            width: 20.0,
            height: 20.0,
        })
        .build();
}
