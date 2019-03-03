use super::physics::Movement;
use crate::physics::{TileMap, TILE_SIZE};
use crate::{Camera, Input};
use quicksilver::geom::{Line, Vector};
use quicksilver::graphics::Color;
use quicksilver::lifecycle::Window;
use specs::prelude::*;

pub struct TileMapRender<'a> {
    pub window: &'a mut Window,
}

impl<'a: 'b, 'b> System<'b> for TileMapRender<'a> {
    type SystemData = (
        Read<'b, TileMap>,
        ReadExpect<'b, Camera>,
        ReadStorage<'b, Movement>,
    );

    fn run(&mut self, (tilemap, camera, movements): Self::SystemData) {
        let camera_pos = camera.get_position(&movements, self.window);
        let screen_size = self.window.screen_size();
        let min_tile_x = (camera_pos.x / TILE_SIZE).floor() as i32;
        let min_tile_y = (camera_pos.y / TILE_SIZE).floor() as i32;
        let max_tile_x = ((camera_pos.x + screen_size.x) / TILE_SIZE).floor() as i32;
        let max_tile_y = ((camera_pos.y + screen_size.y) / TILE_SIZE).floor() as i32;
        for tile_y in min_tile_y..=max_tile_y {
            for tile_x in min_tile_x..=max_tile_x {
                let rect = quicksilver::geom::Rectangle::new(
                    Vector::new(tile_x as f32 * TILE_SIZE, tile_y as f32 * TILE_SIZE) - camera_pos,
                    (TILE_SIZE, TILE_SIZE),
                );
                self.window.draw(
                    &rect,
                    quicksilver::graphics::Background::Col(
                        tilemap
                            .tiles
                            .get(&(tile_x, tile_y))
                            .cloned()
                            .unwrap_or_default()
                            .colour,
                    ),
                );
            }
        }
    }
}

#[derive(Default)]
pub struct RenderComponent {
    pub radius: f32,
    pub colour: Color,
}

impl Component for RenderComponent {
    type Storage = VecStorage<Self>;
}

pub struct Render<'a> {
    pub window: &'a mut Window,
}

impl<'a: 'b, 'b> System<'b> for Render<'a> {
    type SystemData = (
        ReadExpect<'b, Camera>,
        ReadStorage<'b, Movement>,
        ReadStorage<'b, RenderComponent>,
    );

    fn run(&mut self, (camera, movements, render): Self::SystemData) {
        let camera_pos = camera.get_position(&movements, self.window);
        for (movement, render) in (&movements, &render).join() {
            let circle =
                quicksilver::geom::Circle::new(movement.position - camera_pos, render.radius);
            self.window.draw(
                &circle,
                quicksilver::graphics::Background::Col(render.colour),
            );
        }
    }
}

pub struct RenderCursor<'a> {
    pub window: &'a mut Window,
}

impl<'a: 'b, 'b> System<'b> for RenderCursor<'a> {
    type SystemData = (
        ReadExpect<'b, Camera>,
        Read<'b, Input>,
        ReadStorage<'b, Movement>,
    );

    fn run(&mut self, (camera, input, movements): Self::SystemData) {
        let camera_pos = camera.get_position(&movements, self.window);
        let cursor_pos = input.mouse_pos - camera_pos;

        //   |
        //   |
        // -- --
        //   |
        //   |
        // TODO: Figure out what's going on with line rendering to make the cursor symmetrical.
        self.window.draw(
            &Line::new(
                cursor_pos + Vector::new(0, 0),
                cursor_pos + Vector::new(2, 0),
            ),
            quicksilver::graphics::Background::Col(Color::WHITE),
        );
        self.window.draw(
            &Line::new(
                cursor_pos + Vector::new(-1, 0),
                cursor_pos + Vector::new(-3, 0),
            ),
            quicksilver::graphics::Background::Col(Color::WHITE),
        );
        self.window.draw(
            &Line::new(
                cursor_pos + Vector::new(0, 0),
                cursor_pos + Vector::new(0, 2),
            ),
            quicksilver::graphics::Background::Col(Color::WHITE),
        );
        self.window.draw(
            &Line::new(
                cursor_pos + Vector::new(0, -1),
                cursor_pos + Vector::new(0, -3),
            ),
            quicksilver::graphics::Background::Col(Color::WHITE),
        );
    }
}
