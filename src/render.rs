use super::physics::Movement;
use crate::physics::{TileMap, TILE_SIZE};
use quicksilver::graphics::Color;
use quicksilver::lifecycle::Window;
use specs::prelude::*;

pub struct TileMapRender<'a> {
    pub window: &'a mut Window,
}

impl<'a: 'b, 'b> System<'b> for TileMapRender<'a> {
    type SystemData = Read<'b, TileMap>;

    fn run(&mut self, tilemap: Self::SystemData) {
        let screen_size = self.window.screen_size();
        let max_tile_x = (screen_size.x / TILE_SIZE).floor() as i32;
        let max_tile_y = (screen_size.y / TILE_SIZE).floor() as i32;
        for tile_y in 0..=max_tile_y {
            for tile_x in 0..=max_tile_x {
                let rect = quicksilver::geom::Rectangle::new(
                    ((tile_x as f32 * TILE_SIZE), (tile_y as f32 * TILE_SIZE)),
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
    type SystemData = (ReadStorage<'b, Movement>, ReadStorage<'b, RenderComponent>);

    fn run(&mut self, (movement, render): Self::SystemData) {
        for (movement, render) in (&movement, &render).join() {
            let circle = quicksilver::geom::Circle::new(movement.position, render.radius);
            self.window.draw(
                &circle,
                quicksilver::graphics::Background::Col(render.colour),
            );
        }
    }
}
