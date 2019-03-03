use super::{Event, EventQueue};
use quicksilver::geom::{Circle, Rectangle, Shape, Vector};
use quicksilver::graphics::Color;
use specs::prelude::*;
use std::collections::HashMap;

pub const TILE_SIZE: f32 = 32.0;

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

pub struct CollidingWithWall;

impl Component for CollidingWithWall {
    type Storage = HashMapStorage<Self>;
}

pub struct MovementSystem;

impl<'a> System<'a> for MovementSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Movement>,
        ReadStorage<'a, HitBox>,
        Read<'a, TileMap>,
        WriteStorage<'a, CollidingWithWall>,
    );

    fn run(
        &mut self,
        (entities, mut movements, hitboxes, tilemap, mut colliding_with_walls): Self::SystemData,
    ) {
        for (movement, hitbox) in (&mut movements, &hitboxes).join() {
            let new_position = movement.position + movement.velocity;
            let colliding = check_collision(new_position, hitbox, &tilemap);
            if !colliding {
                movement.position = new_position;
            } else {
                movement.velocity = Vector::new(0.0, 0.0);
            }
        }

        for (entity, movement, _) in (&entities, &mut movements, !&hitboxes).join() {
            let new_position = movement.position + movement.velocity;
            movement.position = new_position;

            let colliding = check_point_collision(new_position, &tilemap);
            if colliding {
                colliding_with_walls
                    .insert(entity, CollidingWithWall)
                    .expect("This entity should exists because we just got it from specs");
            } else {
                colliding_with_walls.remove(entity);
            }
        }
    }
}

fn check_point_collision(position: Vector, tilemap: &TileMap) -> bool {
    let tile_x = (position.x / TILE_SIZE).floor() as i32;
    let tile_y = (position.y / TILE_SIZE).floor() as i32;
    tilemap
        .tiles
        .get(&(tile_x, tile_y))
        .cloned()
        .unwrap_or_default()
        .collision
}

fn check_collision(position: Vector, hitbox: &HitBox, tilemap: &TileMap) -> bool {
    let min_x = position.x - hitbox.radius;
    let max_x = position.x + hitbox.radius;
    let min_y = position.y - hitbox.radius;
    let max_y = position.y + hitbox.radius;
    let min_tile_x = (min_x / TILE_SIZE).floor() as i32;
    let min_tile_y = (min_y / TILE_SIZE).floor() as i32;
    let max_tile_x = (max_x / TILE_SIZE).floor() as i32;
    let max_tile_y = (max_y / TILE_SIZE).floor() as i32;
    let hitcircle = Circle::new(position, hitbox.radius);

    for tile_x in min_tile_x..=max_tile_x {
        for tile_y in min_tile_y..=max_tile_y {
            if tilemap
                .tiles
                .get(&(tile_x, tile_y))
                .cloned()
                .unwrap_or_default()
                .collision
                && hitcircle.overlaps(&Rectangle::new(
                    (tile_x as f32 * TILE_SIZE, (tile_y) as f32 * TILE_SIZE),
                    (TILE_SIZE, TILE_SIZE),
                ))
            {
                return true;
            }
        }
    }
    false
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

#[derive(Default)]
pub struct TileMap {
    pub tiles: HashMap<(i32, i32), Tile>,
}

#[derive(Copy, Clone)]
pub struct Tile {
    pub collision: bool,
    pub colour: Color,
}

impl Default for Tile {
    fn default() -> Self {
        Tile {
            collision: false,
            colour: Color::MAGENTA,
        }
    }
}
