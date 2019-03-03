use super::{Event, EventQueue};
use quicksilver::geom::Vector;
use specs::prelude::*;

#[derive(Debug, Default)]
pub struct Movement {
    pub position: Vector,
    pub velocity: Vector,
}

impl Component for Movement {
    type Storage = VecStorage<Self>;
}

#[derive(Debug, Default)]
pub struct Bullet {
    pub radius: f32,
}

impl Component for Bullet {
    type Storage = HashMapStorage<Self>;
}

// Default hitbox is actually a circle.
#[derive(Default)]
pub struct HitBox {
    pub radius: f32,
}

impl Component for HitBox {
    type Storage = VecStorage<Self>;
}

pub struct MovementSystem;

impl<'a> System<'a> for MovementSystem {
    type SystemData = WriteStorage<'a, Movement>;

    fn run(&mut self, mut movement: Self::SystemData) {
        for movement in (&mut movement).join() {
            movement.position += movement.velocity;
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
            for (bullet_movement, bullet_entity, bullet) in (&movements, &entities, &bullet).join()
            {
                if (bullet_movement.position - movement.position).len()
                    < hitbox.radius + bullet.radius
                {
                    event_queue.enqueue(Event::Collision(entity, bullet_entity));
                }
            }
        }
    }
}
