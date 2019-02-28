use super::{Event, EventQueue};
use specs::prelude::*;

#[derive(Debug)]
pub struct Movement {
    pub position: (f32, f32),
    pub velocity: (f32, f32),
}

impl Component for Movement {
    type Storage = VecStorage<Self>;
}

#[derive(Debug, Default)]
pub struct Bullet;

impl Component for Bullet {
    type Storage = HashMapStorage<Self>;
}

pub struct HitBox {
    pub width: f32,
    pub height: f32,
}

impl Component for HitBox {
    type Storage = VecStorage<Self>;
}

pub struct MovementSystem;

impl<'a> System<'a> for MovementSystem {
    type SystemData = WriteStorage<'a, Movement>;

    fn run(&mut self, mut movement: Self::SystemData) {
        for movement in (&mut movement).join() {
            movement.position.0 += movement.velocity.0;
            movement.position.1 += movement.velocity.1;
        }
    }
}

pub struct CollisionDetection;

impl<'a> System<'a> for CollisionDetection {
    type SystemData = (
        ReadStorage<'a, Movement>,
        ReadStorage<'a, HitBox>,
        ReadStorage<'a, Bullet>,
        Entities<'a>,
        Write<'a, EventQueue>,
    );

    fn run(&mut self, (movements, hitbox, bullet, entities, mut event_queue): Self::SystemData) {
        for (movement, hitbox, entity) in (&movements, &hitbox, &entities).join() {
            for (bullet_movement, bullet_entity, _) in (&movements, &entities, &bullet).join() {
                if bullet_movement.position.0 > movement.position.0 - hitbox.width / 2.0
                    && bullet_movement.position.0 < movement.position.0 + hitbox.width / 2.0
                    && bullet_movement.position.1 > movement.position.1 - hitbox.height / 2.0
                    && bullet_movement.position.1 < movement.position.1 + hitbox.height / 2.0
                {
                    event_queue.enqueue(Event::Collision(entity, bullet_entity));
                }
            }
        }
    }
}
