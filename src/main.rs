#![allow(clippy::type_complexity)] // Specs often leads to big SystemData but I think that's fine.

extern crate specs;

use quicksilver::graphics::{Font, FontStyle, Image};
use quicksilver::input::{ButtonState, Key, MouseButton};
use quicksilver::lifecycle::{run, Asset, Settings, State, Window};

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
use physics::{CollisionDetection, PhysicsSystem, TileMap, TILE_SIZE};

mod player;
use player::{PlayerControlSystem, PlayerDeath};

mod gameplay;
use gameplay::{
    BulletSelfDestruct, ChoiceSystem, CollisionHandler, CombativeCollisionHandler, ExitSystem,
};

mod render;
use render::{Render, RenderChoice, RenderCursor, RenderUI, TileMapRender, WorldMapRender};

mod prefabs;
use prefabs::PrefabBuilder;

mod level_generation;
use level_generation::LevelStyle;

mod world_generation;

mod world_map;
use world_map::{CurrentDungeon, Dungeon, WorldMapScreen};

mod enemy_ai;
use enemy_ai::{ChodeDeath, RunChodeAI};

mod all_components {
    pub use crate::enemy_ai::ChodeAI;
    pub use crate::gameplay::{Combative, Destructable, Exit, LevelObject, Team, TeamWrap};
    pub use crate::physics::{Bullet, CollidingWithWall, HitBox, PhysicsComponent, Transform};
    pub use crate::player::PlayerControls;
    pub use crate::render::RenderComponent;
}
use all_components::*;

mod prelude {
    pub use crate::physics::Transform;
    pub use crate::prefabs::PrefabBuilder;
    pub use crate::{SimTime, Timer};
    pub use quicksilver::geom::*;
    pub use quicksilver::graphics::Color;
    pub use rand::Rng;
    pub use specs::*;
}
use prelude::*;

#[derive(Copy, Clone, Debug, Default)]
pub struct SimTime {
    time: f32,
    dt: f32,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Timer {
    expire_time: f32,
}

#[derive(Default)]
pub struct PlayerProgression {
    pub range_extended: bool,
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
    dodge: bool,
    raw_mouse_pos: Vector,
    mouse_pos: Vector,
    clicked: bool,
}

#[derive(Debug, Copy, Clone)]
pub enum Event {
    Collision(Entity, Entity),
    EntityKilled(Entity),
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

#[derive(Clone)]
pub enum UIState {
    Title,
    WorldMap,
    Playing,
    Pause,
    GameOver,
    Victory,
    BossFight,
    Choice,
}

impl Default for UIState {
    fn default() -> Self {
        UIState::Title
    }
}

struct GameState {
    world: World,
    dispatcher: Dispatcher<'static, 'static>,
    font: Font,
    title_image: Asset<Image>,
}

pub struct Camera {
    follow: Entity,
}

impl Camera {
    fn get_position<'a>(&self, movements: &ReadStorage<'a, Transform>, window: &Window) -> Vector {
        movements
            .get(self.follow)
            .expect("TODO: Remember where the camera was last and don't crash")
            .position
            - window.screen_size() / 2.0
    }
}

#[derive(Default)]
pub struct ScreenSize {
    pub size: Vector,
}

impl State for GameState {
    fn new() -> quicksilver::Result<Self> {
        let title_image = Asset::new(Image::load("title.png"));
        let level = level_generation::generate_level(LevelStyle::Cyclic);

        let font =
            Font::from_slice(include_bytes!("fonts/fonts/OpenSans/OpenSans-Regular.ttf")).unwrap();

        let mut world = World::new();

        world.register::<Transform>();
        world.register::<PhysicsComponent>();
        world.register::<PlayerControls>();
        world.register::<RenderComponent>();
        world.register::<HitBox>();
        world.register::<Bullet>();
        world.register::<CollidingWithWall>();
        world.register::<Dungeon>();
        world.register::<Exit>();
        world.register::<Destructable>();
        world.register::<LevelObject>();
        world.register::<Combative>();
        world.register::<ChodeAI>();
        world.register::<TeamWrap>();

        let player = world
            .create_entity()
            .with_player_prefab()
            .with(Transform {
                position: Vector::from(level.start_position) * TILE_SIZE
                    + Vector::new(TILE_SIZE / 2.0, TILE_SIZE / 2.0),
            })
            .build();
        world
            .create_entity()
            .with_target_prefab()
            .with(Transform {
                position: Vector::new(SCREEN_WIDTH / 2.0, 100.0),
            })
            .build();
        world.add_resource::<Input>(Default::default());
        world.add_resource::<SimTime>(Default::default());
        world.add_resource::<EventQueue>(Default::default());
        world.add_resource::<TileMap>(level.tile_map);
        world.add_resource(Camera { follow: player });
        world.add_resource(UIState::Title);
        world.add_resource::<ScreenSize>(Default::default());
        world.add_resource::<PlayerProgression>(Default::default());
        world.add_resource::<CurrentDungeon>(Default::default());

        world_generation::generate_dungeons(&mut world);
        Ok(GameState {
            world,
            dispatcher: make_dispatcher(),
            font,
            title_image,
        })
    }

    fn update(&mut self, window: &mut Window) -> quicksilver::Result<()> {
        let input = Input {
            down: window.keyboard()[Key::S].is_down(),
            left: window.keyboard()[Key::A].is_down(),
            up: window.keyboard()[Key::W].is_down(),
            right: window.keyboard()[Key::D].is_down(),
            fire: window.mouse()[MouseButton::Left].is_down(),
            dodge: window.mouse()[MouseButton::Right].is_down()
                || window.keyboard()[Key::LShift].is_down(),
            raw_mouse_pos: window.mouse().pos(),
            mouse_pos: window.mouse().pos()
                + self
                    .world
                    .read_resource::<Camera>()
                    .get_position(&self.world.read_storage(), window),
            clicked: window.mouse()[MouseButton::Left] == ButtonState::Pressed,
        };
        self.world.add_resource(input);
        self.world.add_resource(ScreenSize {
            size: window.screen_size(),
        });

        let ui_state = (*self.world.read_resource::<UIState>()).clone();
        match ui_state {
            UIState::Title => {
                if window.keyboard()[Key::Escape] == ButtonState::Pressed {
                    window.close();
                }
                if window.keyboard()[Key::Space] == ButtonState::Pressed {
                    self.world.add_resource(UIState::WorldMap);
                }
                Ok(())
            }
            UIState::WorldMap => {
                use specs::RunNow;
                let mut world_map_screen = WorldMapScreen;
                world_map_screen.run_now(&self.world.res);
                Ok(())
            }
            UIState::Playing => {
                // Noclip mode, a bit hacky.
                if window.keyboard()[Key::N] == ButtonState::Pressed {
                    let player = self.world.read_resource::<Camera>().follow;
                    if self.world.read_storage::<HitBox>().get(player).is_some() {
                        self.world.write_storage::<HitBox>().remove(player);
                    } else {
                        self.world
                            .write_storage::<HitBox>()
                            .insert(player, HitBox { radius: 15.0 })
                            .expect("Player should be alive"); // TODO Don't hardcode radius
                    }
                }

                let mut sim_time = *self.world.read_resource::<SimTime>();
                sim_time.time += 1.0 / 60.0; // Quicksilver tries to call at 60fps
                sim_time.dt = 1.0 / 60.0;
                self.world.add_resource(sim_time);
                self.world.write_resource::<EventQueue>().clear();
                self.dispatcher.dispatch(&self.world.res);
                self.world.maintain();
                Ok(())
            }
            UIState::Victory | UIState::GameOver => {
                if window.keyboard()[Key::Space] == ButtonState::Pressed
                    || window.keyboard()[Key::Escape] == ButtonState::Pressed
                {
                    self.world.add_resource(UIState::Title);
                }
                Ok(())
            }
            UIState::Choice => {
                ChoiceSystem.run_now(&self.world.res);
                Ok(())
            }

            _ => panic!("Unimplemented ui state"),
        }
    }

    fn draw(&mut self, window: &mut Window) -> quicksilver::Result<()> {
        use specs::RunNow;

        window.clear(quicksilver::graphics::Color::BLACK).unwrap();

        match self.world.read_resource::<UIState>().clone() {
            UIState::Title => {
                self.title_image.execute(|image| {
                    window.draw(
                        &image.area().with_center((400, 300)),
                        quicksilver::graphics::Background::Img(&image),
                    );
                    Ok(())
                })?;
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
                let mut render_ui = RenderUI {
                    window,
                    font: &self.font,
                };
                render_ui.run_now(&self.world.res);
                let mut render_cursor = RenderCursor { window };
                render_cursor.run_now(&self.world.res);
                Ok(())
            }
            UIState::Victory => {
                draw_text_centered("YOU WIN! :)", Vector::new(400, 300), &self.font, window);
                draw_text_centered(
                    "Space or Esc to return to title",
                    Vector::new(400, 350),
                    &self.font,
                    window,
                );
                Ok(())
            }
            UIState::GameOver => {
                draw_text_centered("You died. D:", Vector::new(400, 300), &self.font, window);
                draw_text_centered(
                    "Space or Esc to return to title",
                    Vector::new(400, 350),
                    &self.font,
                    window,
                );
                Ok(())
            }
            UIState::Choice => {
                let mut render_choice = RenderChoice {
                    window,
                    font: &self.font,
                };
                render_choice.run_now(&self.world.res);
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
        .with(RunChodeAI, "run_chode_ai", &[])
        .with(
            PhysicsSystem,
            "physics",
            &["player_control", "run_chode_ai"],
        )
        .with(CollisionDetection, "collision_detection", &["physics"])
        .with(
            CollisionHandler,
            "collision_handler",
            &["collision_detection"],
        )
        .with(
            CombativeCollisionHandler,
            "combative_collision_handler",
            &["collision_detection"],
        )
        .with(ChodeDeath, "chode_death", &["combative_collision_handler"])
        .with(
            PlayerDeath,
            "player_death",
            &["combative_collision_handler"],
        )
        .with(BulletSelfDestruct, "bullet_self_destruct", &["physics"])
        .with(ExitSystem, "exit", &["physics"])
        .build()
}

fn main() {
    run::<GameState>(
        "Specs Test",
        quicksilver::geom::Vector::new(SCREEN_WIDTH, SCREEN_HEIGHT),
        Settings::default(),
    );
}
