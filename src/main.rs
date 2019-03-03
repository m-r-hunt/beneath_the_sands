extern crate specs;

use quicksilver::geom::Vector;
use quicksilver::graphics::Color;
use quicksilver::input::{ButtonState, MouseButton};
use quicksilver::lifecycle::{run, Settings, State, Window};
use specs::{Builder, Dispatcher, DispatcherBuilder, Entity, World};

const SCREEN_WIDTH: f32 = 800.0;
const SCREEN_HEIGHT: f32 = 600.0;

mod physics;
use physics::{Bullet, CollisionDetection, HitBox, Movement, MovementSystem, Tile, TileMap};

mod player;
use player::{PlayerControlSystem, PlayerControls};

mod gameplay;
use gameplay::CollisionHandler;

mod render;
use render::{Render, RenderComponent, TileMapRender};

mod prefabs;
use prefabs::PrefabBuilder;

mod all_components {
    pub use crate::physics::{Bullet, HitBox, Movement};
    pub use crate::player::PlayerControls;
    pub use crate::render::RenderComponent;
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
    Playing,
    Pause(Dispatcher<'static, 'static>),
    GameOver(Dispatcher<'static, 'static>),
}

struct GameState {
    ui_state: UIState,
    world: World,
    dispatcher: Dispatcher<'static, 'static>,
}

impl State for GameState {
    fn new() -> quicksilver::Result<Self> {
        let mut world = World::new();
        world.register::<Movement>();
        world.register::<PlayerControls>();
        world.register::<RenderComponent>();
        world.register::<HitBox>();
        world.register::<Bullet>();
        world
            .create_entity()
            .with_player_prefab()
            .with(Movement {
                position: Vector::new(SCREEN_WIDTH / 2.0, SCREEN_HEIGHT - 100.0),
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
        world.add_resource::<TileMap>(Default::default());
        world.write_resource::<TileMap>().tiles.insert(
            (0, 0),
            Tile {
                collision: true,
                colour: Color::WHITE,
            },
        );
        Ok(GameState {
            ui_state: UIState::Playing,
            world,
            dispatcher: make_dispatcher(),
        })
    }

    fn update(&mut self, window: &mut Window) -> quicksilver::Result<()> {
        match self.ui_state {
            UIState::Title => {
                if window.keyboard()[quicksilver::input::Key::Escape] == ButtonState::Pressed {
                    window.close();
                }
                if window.keyboard()[quicksilver::input::Key::Space] == ButtonState::Pressed {
                    self.ui_state = UIState::Playing;
                }
                Ok(())
            }
            UIState::Playing => {
                let input = Input {
                    down: window.keyboard()[quicksilver::input::Key::S].is_down(),
                    left: window.keyboard()[quicksilver::input::Key::A].is_down(),
                    up: window.keyboard()[quicksilver::input::Key::W].is_down(),
                    right: window.keyboard()[quicksilver::input::Key::D].is_down(),
                    fire: window.mouse()[MouseButton::Left].is_down(),
                    mouse_pos: window.mouse().pos(),
                };
                self.world.add_resource(input);
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

        match self.ui_state {
            UIState::Title => Ok(()),
            UIState::Playing => {
                window.clear(quicksilver::graphics::Color::BLACK).unwrap();
                let mut tilemap_render = TileMapRender { window };
                tilemap_render.run_now(&self.world.res);
                let mut render = Render { window };
                render.run_now(&self.world.res);
                Ok(())
            }
            _ => panic!("Unimplented ui state"),
        }
    }
}

fn make_dispatcher<'a, 'b>() -> Dispatcher<'a, 'b> {
    DispatcherBuilder::new()
        .with(PlayerControlSystem, "player_control", &[])
        .with(MovementSystem, "movement", &["player_control"])
        .with(CollisionDetection, "collision_detection", &[])
        .with(
            CollisionHandler,
            "collision_handler",
            &["collision_detection"],
        )
        .build()
}

fn main() {
    run::<GameState>(
        "Specs Test",
        quicksilver::geom::Vector::new(SCREEN_WIDTH, SCREEN_HEIGHT),
        Settings::default(),
    );
}
