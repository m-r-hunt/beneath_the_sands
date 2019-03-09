use crate::enemy_ai::Boss;
use crate::gameplay::Combative;
use crate::physics::{TileMap, TILE_SIZE};
use crate::player::PlayerControls;
use crate::prelude::*;
use crate::world_map::{Dungeon, Reward, RANGE1, RANGE2};
use crate::{draw_text_centered, Camera, CurrentDungeon, Input, PlayerProgression};
use quicksilver::graphics::Font;
use quicksilver::lifecycle::Window;

pub struct TileMapRender<'a> {
    pub window: &'a mut Window,
}

impl<'a: 'b, 'b> System<'b> for TileMapRender<'a> {
    type SystemData = (
        Read<'b, TileMap>,
        ReadStorage<'b, Camera>,
        ReadStorage<'b, Transform>,
    );

    fn run(&mut self, (tilemap, camera, transforms): Self::SystemData) {
        let mut camera_pos = Vector::new(-1.0, -1.0);
        for (_, camera_transform) in (&camera, &transforms).join() {
            camera_pos = camera_transform.position;
        }
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
        ReadStorage<'b, Camera>,
        ReadStorage<'b, Transform>,
        ReadStorage<'b, RenderComponent>,
    );

    fn run(&mut self, (camera, transforms, render): Self::SystemData) {
        let mut camera_pos = Vector::new(-1.0, -1.0);
        for (_, camera_transform) in (&camera, &transforms).join() {
            camera_pos = camera_transform.position;
        }
        for (movement, render) in (&transforms, &render).join() {
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
    type SystemData = (
        ReadStorage<'b, PlayerControls>,
        ReadStorage<'b, Combative>,
        ReadStorage<'b, Boss>,
    );

    fn run(&mut self, (players, combative, bosses): Self::SystemData) {
        for (_, c) in (&players, &combative).join() {
            draw_text_centered(
                &format!("Health: {} / {}", c.max_hp - c.damage, c.max_hp),
                Vector::new(100, 25),
                &self.font,
                self.window,
            );
        }
        for (_, c) in (&bosses, &combative).join() {
            draw_text_centered(
                &format!("Boss Health: {} / {}", c.max_hp - c.damage, c.max_hp),
                Vector::new(100, 50),
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
    type SystemData = (Read<'b, Input>,);

    fn run(&mut self, (input,): Self::SystemData) {
        draw_cursor(input.raw_mouse_pos, self.window);
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
                } else if let Reward::Progress = d.reward {
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

pub struct RenderChoice<'a> {
    pub window: &'a mut Window,
    pub font: &'a Font,
}

impl<'a: 'b, 'b> System<'b> for RenderChoice<'a> {
    type SystemData = (Read<'b, CurrentDungeon>, ReadStorage<'b, Dungeon>);

    fn run(&mut self, (current_dungeon, dungeons): Self::SystemData) {
        let current_dungeon = current_dungeon
            .entity
            .expect("We should be playing a dungeon when we hit are doing choice.");
        let current_dungeon = dungeons
            .get(current_dungeon)
            .expect("The current dungeon should be valid when are doing choice.");
        if let Reward::Choice(item1, item2) = current_dungeon.reward {
            draw_text_centered(
                "Choose Upgrade:",
                Vector::new(400, 150),
                &self.font,
                self.window,
            );
            draw_text_centered(
                &format!("{:?}", item1),
                Vector::new(150, 300),
                &self.font,
                self.window,
            );
            draw_text_centered(
                &format!("{:?}", item2),
                Vector::new(650, 300),
                &self.font,
                self.window,
            );
        } else {
            panic!("Bad choice state");
        }
    }
}
