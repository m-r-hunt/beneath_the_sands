use crate::gameplay::Combative;
use crate::physics::{TileMap, TILE_SIZE};
use crate::player::PlayerControls;
use crate::prelude::*;
use crate::world_map::{Dungeon, RANGE1, RANGE2};
use crate::{draw_text_centered, Camera, Input, PlayerProgression};
use quicksilver::graphics::Font;
use quicksilver::lifecycle::Window;

pub struct TileMapRender<'a> {
    pub window: &'a mut Window,
}

impl<'a: 'b, 'b> System<'b> for TileMapRender<'a> {
    type SystemData = (
        Read<'b, TileMap>,
        ReadExpect<'b, Camera>,
        ReadStorage<'b, Transform>,
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
        ReadStorage<'b, Transform>,
        ReadStorage<'b, RenderComponent>,
    );

    fn run(&mut self, (camera, movements, render): Self::SystemData) {
        let camera_pos = camera.get_position(&movements, self.window);
        for (movement, render) in (&movements, &render).join() {
            let circle = Circle::new(movement.position - camera_pos, render.radius);
            self.window.draw(
                &circle,
                quicksilver::graphics::Background::Col(render.colour),
            );
        }
    }
}

pub struct RenderUI<'a> {
    pub window: &'a mut Window,
    pub font: &'a Font,
}

impl<'a: 'b, 'b> System<'b> for RenderUI<'a> {
    type SystemData = (ReadStorage<'b, PlayerControls>, ReadStorage<'b, Combative>);

    fn run(&mut self, (players, combative): Self::SystemData) {
        for (_, c) in (&players, &combative).join() {
            draw_text_centered(
                &format!("Health: {} / {}", c.max_hp - c.damage, c.max_hp),
                Vector::new(50, 25),
                &self.font,
                self.window,
            );
        }
    }
}

pub struct RenderCursor<'a> {
    pub window: &'a mut Window,
}

fn draw_cursor(cursor_pos: Vector, window: &mut Window) {
    //   |
    //   |
    // -- --
    //   |
    //   |
    // TODO: Figure out what's going on with line rendering to make the cursor symmetrical.
    window.draw(
        &Line::new(
            cursor_pos + Vector::new(0, 0),
            cursor_pos + Vector::new(2, 0),
        ),
        quicksilver::graphics::Background::Col(Color::WHITE),
    );
    window.draw(
        &Line::new(
            cursor_pos + Vector::new(-1, 0),
            cursor_pos + Vector::new(-3, 0),
        ),
        quicksilver::graphics::Background::Col(Color::WHITE),
    );
    window.draw(
        &Line::new(
            cursor_pos + Vector::new(0, 0),
            cursor_pos + Vector::new(0, 2),
        ),
        quicksilver::graphics::Background::Col(Color::WHITE),
    );
    window.draw(
        &Line::new(
            cursor_pos + Vector::new(0, -1),
            cursor_pos + Vector::new(0, -3),
        ),
        quicksilver::graphics::Background::Col(Color::WHITE),
    );
}

impl<'a: 'b, 'b> System<'b> for RenderCursor<'a> {
    type SystemData = (
        ReadExpect<'b, Camera>,
        Read<'b, Input>,
        ReadStorage<'b, Transform>,
    );

    fn run(&mut self, (camera, input, movements): Self::SystemData) {
        let camera_pos = camera.get_position(&movements, self.window);
        let cursor_pos = input.mouse_pos - camera_pos;
        draw_cursor(cursor_pos, self.window);
    }
}

pub struct WorldMapRender<'a> {
    pub window: &'a mut Window,
}

impl<'a: 'b, 'b> System<'b> for WorldMapRender<'a> {
    type SystemData = (
        Read<'b, Input>,
        ReadStorage<'b, Dungeon>,
        Read<'b, PlayerProgression>,
    );

    fn run(&mut self, (input, dungeons, progress): Self::SystemData) {
        let screen_size = self.window.screen_size();
        let offset = screen_size / 2.0;

        // Draw oasis
        let circle = Circle::new(offset, 10.0);
        self.window
            .draw(&circle, quicksilver::graphics::Background::Col(Color::BLUE));

        for d in dungeons.join() {
            let rect = Rectangle::new(d.position + offset, Vector::new(10.0, 10.0));
            self.window.draw(
                &rect,
                quicksilver::graphics::Background::Col(if d.completed {
                    Color::GREEN
                } else if d.reward {
                    Color::ORANGE
                } else {
                    Color::RED
                }),
            );
        }

        let circle = Circle::new(
            offset,
            if progress.range_extended {
                RANGE2
            } else {
                RANGE1
            },
        );
        self.window.draw(
            &circle,
            quicksilver::graphics::Background::Col(rgba!(0.0, 0.0, 250.0, 0.25)),
        );

        draw_cursor(input.raw_mouse_pos, self.window);
    }
}
