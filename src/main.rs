extern crate specs;

use quicksilver::geom::Vector;
use quicksilver::graphics::{Color, Font, FontStyle};
use quicksilver::input::{ButtonState, MouseButton};
use quicksilver::lifecycle::{run, Settings, State, Window};
use specs::prelude::*;

const SCREEN_WIDTH: f32 = 800.0;
const SCREEN_HEIGHT: f32 = 600.0;

macro_rules! rgba {
    ($r:expr, $g:expr, $b: expr, $a: expr) => {
        quicksilver::graphics::Color {
            r: $r as f32 / 255.0,
            g: $g as f32 / 255.0,
            b: $b as f32 / 255.0,
            a: $a,
        }
    };
}

mod physics;
use physics::{
    Bullet, CollidingWithWall, CollisionDetection, HitBox, Movement, MovementSystem, TileMap,
    TILE_SIZE,
};

mod player;
use player::{PlayerControlSystem, PlayerControls};

mod gameplay;
use gameplay::{BulletSelfDestruct, CollisionHandler};

mod render;
use render::{Render, RenderComponent, RenderCursor, TileMapRender, WorldMapRender};

mod prefabs;
use prefabs::PrefabBuilder;

mod level_generation;

mod world_generation;
use world_generation::Dungeon;

mod all_components {
    pub use crate::physics::{Bullet, HitBox, Movement};
    pub use crate::player::PlayerControls;
    pub use crate::render::RenderComponent;
}

mod prelude {
    pub use crate::physics::Movement;
    pub use quicksilver::geom::*;
    pub use quicksilver::graphics::Color;
    pub use rand::Rng;
    pub use specs::*;
}

#[derive(Copy, Clone, Debug, Default)]
pub struct SimTime {
    time: f32,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Timer {
    expire_time: f32,
}

impl Timer {
    #[allow(dead_code)]
    fn new_set(sim_time: SimTime, duration: f32) -> Timer {
        Timer {
            expire_time: sim_time.time + duration,
        }
    }

    fn set(&mut self, sim_time: SimTime, duration: f32) {
        self.expire_time = sim_time.time + duration;
    }

    fn expired(self, sim_time: SimTime) -> bool {
        sim_time.time > self.expire_time
    }
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct Input {
    left: bool,
    right: bool,
    up: bool,
    down: bool,
    fire: bool,
    raw_mouse_pos: Vector,
    mouse_pos: Vector,
}

#[derive(Debug, Copy, Clone)]
pub enum Event {
    Collision(Entity, Entity),
}

#[derive(Debug, Default, Clone)]
pub struct EventQueue {
    events: Vec<Event>,
}

impl EventQueue {
    fn clear(&mut self) {
        self.events.clear();
    }

    fn enqueue(&mut self, event: Event) {
        self.events.push(event);
    }

    fn iter(&self) -> impl Iterator<Item = &Event> {
        self.events.iter()
    }
}

enum UIState {
    Title,
    WorldMap,
    Playing,
    Pause(Dispatcher<'static, 'static>),
    GameOver(Dispatcher<'static, 'static>),
}

struct GameState {
    ui_state: UIState,
    world: World,
    dispatcher: Dispatcher<'static, 'static>,
    font: Font,
}

pub struct Camera {
    follow: Entity,
}

impl Camera {
    fn get_position<'a>(&self, movements: &ReadStorage<'a, Movement>, window: &Window) -> Vector {
        movements
            .get(self.follow)
            .expect("TODO: Remember where the camera was last and don't crash")
            .position
            - window.screen_size() / 2.0
    }
}

impl State for GameState {
    fn new() -> quicksilver::Result<Self> {
        let level = level_generation::generate_level();

        let font =
            Font::from_slice(include_bytes!("fonts/fonts/OpenSans/OpenSans-Regular.ttf")).unwrap();

        let mut world = World::new();

        world.register::<Movement>();
        world.register::<PlayerControls>();
        world.register::<RenderComponent>();
        world.register::<HitBox>();
        world.register::<Bullet>();
        world.register::<CollidingWithWall>();
        world.register::<Dungeon>();

        let player = world
            .create_entity()
            .with_player_prefab()
            .with(Movement {
                position: Vector::from(level.start_position) * TILE_SIZE,
                velocity: Vector::new(0.0, 0.0),
            })
            .build();
        world
            .create_entity()
            .with_target_prefab()
            .with(Movement {
                position: Vector::new(SCREEN_WIDTH / 2.0, 100.0),
                velocity: Vector::new(0.0, 0.0),
            })
            .build();
        world.add_resource::<Input>(Default::default());
        world.add_resource::<SimTime>(Default::default());
        world.add_resource::<EventQueue>(Default::default());
        world.add_resource::<TileMap>(level.tile_map);
        world.add_resource(Camera { follow: player });

        world_generation::generate_dungeons(&mut world);
        Ok(GameState {
            ui_state: UIState::Title,
            world,
            dispatcher: make_dispatcher(),
            font,
        })
    }

    fn update(&mut self, window: &mut Window) -> quicksilver::Result<()> {
        let input = Input {
            down: window.keyboard()[quicksilver::input::Key::S].is_down(),
            left: window.keyboard()[quicksilver::input::Key::A].is_down(),
            up: window.keyboard()[quicksilver::input::Key::W].is_down(),
            right: window.keyboard()[quicksilver::input::Key::D].is_down(),
            fire: window.mouse()[MouseButton::Left].is_down(),
            raw_mouse_pos: window.mouse().pos(),
            mouse_pos: window.mouse().pos()
                + self
                    .world
                    .read_resource::<Camera>()
                    .get_position(&self.world.read_storage(), window),
        };
        self.world.add_resource(input);

        match self.ui_state {
            UIState::Title => {
                if window.keyboard()[quicksilver::input::Key::Escape] == ButtonState::Pressed {
                    window.close();
                }
                if window.keyboard()[quicksilver::input::Key::Space] == ButtonState::Pressed {
                    self.ui_state = UIState::WorldMap;
                }
                Ok(())
            }
            UIState::WorldMap => Ok(()),
            UIState::Playing => {
                // Noclip mode, a bit hacky.
                if window.keyboard()[quicksilver::input::Key::N] == ButtonState::Pressed {
                    let player = self.world.read_resource::<Camera>().follow;
                    if self.world.read_storage::<HitBox>().get(player).is_some() {
                        self.world.write_storage::<HitBox>().remove(player);
                    } else {
                        self.world
                            .write_storage::<HitBox>()
                            .insert(player, HitBox { radius: 20.0 })
                            .expect("Player should be alive"); // TODO Don't hardcode radius
                    }
                }

                let mut sim_time = *self.world.read_resource::<SimTime>();
                sim_time.time += 1.0 / 60.0; // Quicksilver tries to call at 60fps
                self.world.add_resource(sim_time);
                self.world.write_resource::<EventQueue>().clear();
                self.dispatcher.dispatch_seq(&self.world.res);
                self.world.maintain();
                Ok(())
            }
            _ => panic!("Unimplemented ui state"),
        }
    }

    fn draw(&mut self, window: &mut Window) -> quicksilver::Result<()> {
        use specs::RunNow;

        window.clear(quicksilver::graphics::Color::BLACK).unwrap();

        match self.ui_state {
            UIState::Title => {
                draw_text_centered(
                    "Beneath The Sands",
                    Vector::new(400, 300),
                    &self.font,
                    window,
                );
                draw_text_centered("Space to Start", Vector::new(400, 350), &self.font, window);
                draw_text_centered("Esc to Quit", Vector::new(400, 400), &self.font, window);
                Ok(())
            }
            UIState::WorldMap => {
                let mut world_map_render = WorldMapRender { window };
                world_map_render.run_now(&self.world.res);
                Ok(())
            }
            UIState::Playing => {
                let mut tilemap_render = TileMapRender { window };
                tilemap_render.run_now(&self.world.res);
                let mut render = Render { window };
                render.run_now(&self.world.res);
                let mut render_cursor = RenderCursor { window };
                render_cursor.run_now(&self.world.res);
                Ok(())
            }
            _ => panic!("Unimplented ui state"),
        }
    }
}

fn draw_text_centered(text: &str, position: Vector, font: &Font, window: &mut Window) {
    let img = font
        .render(text, &FontStyle::new(32.0, Color::WHITE))
        .unwrap();
    let mut rect = img.area();
    rect.pos = position - rect.size / 2.0;
    window.draw(&rect, quicksilver::graphics::Background::Img(&img));
}

fn make_dispatcher<'a, 'b>() -> Dispatcher<'a, 'b> {
    DispatcherBuilder::new()
        .with(PlayerControlSystem, "player_control", &[])
        .with(MovementSystem, "movement", &["player_control"])
        .with(CollisionDetection, "collision_detection", &["movement"])
        .with(
            CollisionHandler,
            "collision_handler",
            &["collision_detection"],
        )
        .with(BulletSelfDestruct, "bullet_self_destruct", &["movement"])
        .build()
}

fn main() {
    run::<GameState>(
        "Specs Test",
        quicksilver::geom::Vector::new(SCREEN_WIDTH, SCREEN_HEIGHT),
        Settings::default(),
    );
}
