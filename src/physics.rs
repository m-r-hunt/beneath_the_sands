use crate::prelude::*;
use crate::{Event, EventQueue};
use std::collections::HashMap;

fn sign(a: i32) -> i32 {
    if a > 0 {
        1
    } else if a < 0 {
        -1
    } else {
        0
    }
}

pub const TILE_SIZE: f32 = 32.0;

#[derive(Debug, Default)]
pub struct Transform {
    pub position: Vector,
}

#[derive(Debug, Default)]
pub struct PhysicsComponent {
    pub velocity: Vector,
    pub acceleration: Vector,
    pub max_speed: f32,
}

impl Component for PhysicsComponent {
    type Storage = DenseVecStorage<Self>;
}

impl Component for Transform {
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

pub struct PhysicsSystem;

impl<'a> System<'a> for PhysicsSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Transform>,
        WriteStorage<'a, PhysicsComponent>,
        ReadStorage<'a, HitBox>,
        Read<'a, TileMap>,
        WriteStorage<'a, CollidingWithWall>,
    );

    fn run(
        &mut self,
        (entities, mut transforms, mut physics, hitboxes, tilemap, mut colliding_with_walls): Self::SystemData,
    ) {
        for physics in (&mut physics).join() {
            physics.velocity += physics.acceleration;
            if physics.velocity.len2() >= physics.max_speed * physics.max_speed {
                physics.velocity = physics.velocity.with_len(physics.max_speed);
            }
        }

        for (transform, physics, hitbox) in (&mut transforms, &mut physics, &hitboxes).join() {
            let round_position = (transform.position.x.floor(), transform.position.y.floor());
            assert!(!check_collision(
                Vector::from(round_position),
                hitbox,
                &tilemap
            ));
            let new_position = transform.position + physics.velocity;
            let old_x = transform.position.x.floor() as i32;
            let new_x = new_position.x.floor() as i32;
            let dx = sign(new_x - old_x);
            let steps = (new_x - old_x).abs();
            let mut hit = false;
            for ix in 0..steps {
                let x = old_x + (1 + ix) * dx;
                let colliding =
                    check_collision(Vector::new(x as f32, round_position.1), hitbox, &tilemap);
                if !colliding {
                    transform.position.x = x as f32;
                } else {
                    physics.velocity.x = 0.0;
                    hit = true;
                    break;
                }
            }
            if !hit {
                transform.position.x = new_position.x;
            }
            let round_position = (transform.position.x.floor(), transform.position.y.floor());
            let old_y = transform.position.y.floor() as i32;
            let new_y = new_position.y.floor() as i32;
            let dy = sign(new_y - old_y);
            let steps = (new_y - old_y).abs();
            let mut hit = false;
            for iy in 0..steps {
                let y = old_y + (1 + iy) * dy;
                let colliding =
                    check_collision(Vector::new(round_position.0, y as f32), hitbox, &tilemap);
                if !colliding {
                    transform.position.y = y as f32;
                } else {
                    physics.velocity.y = 0.0;
                    hit = true;
                    break;
                }
            }
            if !hit {
                transform.position.y = new_position.y;
            }
            assert!(!check_collision(
                Vector::new(transform.position.x.floor(), transform.position.y.floor()),
                hitbox,
                &tilemap
            ));
        }

        for (entity, transform, physics, _) in
            (&entities, &mut transforms, &physics, !&hitboxes).join()
        {
            let new_position = transform.position + physics.velocity;
            transform.position = new_position;

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

pub fn check_collision(position: Vector, hitbox: &HitBox, tilemap: &TileMap) -> bool {
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
        ReadStorage<'a, Transform>,
        ReadStorage<'a, HitBox>,
        ReadStorage<'a, Bullet>,
        Entities<'a>,
        Write<'a, EventQueue>,
    );

    fn run(&mut self, (transforms, hitbox, bullet, entities, mut event_queue): Self::SystemData) {
        for (transform, hitbox, entity) in (&transforms, &hitbox, &entities).join() {
            for (bullet_transform, bullet_entity, bullet) in
                (&transforms, &entities, &bullet).join()
            {
                if (bullet_transform.position - transform.position).len()
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

pub fn hitbox_overlap(
    transform1: &Transform,
    hitbox1: &HitBox,
    transform2: &Transform,
    hitbox2: &HitBox,
) -> bool {
    let c1 = Circle::new(transform1.position, hitbox1.radius);
    let c2 = Circle::new(transform2.position, hitbox2.radius);
    c1.overlaps(&c2)
}
