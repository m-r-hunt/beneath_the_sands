extern crate specs;

use quicksilver::graphics::Color;
use quicksilver::lifecycle::{run, Settings, State, Window};
use specs::{Builder, Dispatcher, DispatcherBuilder, Entity, World};

const SCREEN_WIDTH: f32 = 250.0;
const SCREEN_HEIGHT: f32 = 600.0;

mod physics;
use physics::{Bullet, CollisionDetection, HitBox, Movement, MovementSystem};

mod player;
use player::{PlayerControlSystem, PlayerControls, SoftBoundsCheck};

mod gameplay;
use gameplay::{
    spawn_chode, CollisionHandler, HardBoundsCheck, Spawned, Spawner, SpawnerState, Wave,
};

mod render;
use render::{Render, RenderComponent};

#[derive(Copy, Clone, Debug, Default)]
pub struct SimTime {
    time: f32,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Timer {
    expire_time: f32,
}

impl Timer {
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
}

#[derive(Default)]
pub struct WorldBounds {
    top: f32,
    left: f32,
    bottom: f32,
    right: f32,
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

struct GameState {
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
        world.register::<Spawned>();
        world
            .create_entity()
            .with(Movement {
                position: (SCREEN_WIDTH / 2.0, 100.0),
                velocity: (0.0, 0.0),
            })
            .with(HitBox {
                width: 50.0,
                height: 50.0,
            })
            .with(RenderComponent {
                width: 50.0,
                height: 50.0,
                colour: Color::RED,
            })
            .build();
        world
            .create_entity()
            .with(Movement {
                position: (SCREEN_WIDTH / 2.0, SCREEN_HEIGHT - 100.0),
                velocity: (0.0, 0.0),
            })
            .with(RenderComponent {
                width: 20.0,
                height: 50.0,
                colour: Color::BLUE,
            })
            .with(PlayerControls {
                fire_cooldown: Default::default(),
            })
            .build();
        world.add_resource::<Input>(Default::default());
        world.add_resource::<SimTime>(Default::default());
        world.add_resource::<EventQueue>(Default::default());
        world.add_resource(WorldBounds {
            top: 0.0,
            left: 0.0,
            right: SCREEN_WIDTH,
            bottom: SCREEN_HEIGHT,
        });
        Ok(GameState {
            world,
            dispatcher: make_dispatcher(),
        })
    }

    fn update(&mut self, window: &mut Window) -> quicksilver::Result<()> {
        let input = Input {
            down: window.keyboard()[quicksilver::input::Key::S].is_down(),
            left: window.keyboard()[quicksilver::input::Key::A].is_down(),
            up: window.keyboard()[quicksilver::input::Key::W].is_down(),
            right: window.keyboard()[quicksilver::input::Key::D].is_down(),
            fire: window.keyboard()[quicksilver::input::Key::Space].is_down(),
        };
        self.world.add_resource(input);
        let mut sim_time = *self.world.read_resource::<SimTime>();
        sim_time.time += 1.0 / 60.0; // Quicksilver tries to call at 60fps
        self.world.add_resource(sim_time);
        self.world.write_resource::<EventQueue>().clear();
        self.dispatcher.dispatch(&self.world.res);
        self.world.maintain();
        Ok(())
    }

    fn draw(&mut self, window: &mut Window) -> quicksilver::Result<()> {
        use specs::RunNow;

        let mut render = Render { window };
        render.run_now(&self.world.res);
        Ok(())
    }
}

fn make_dispatcher<'a, 'b>() -> Dispatcher<'a, 'b> {
    DispatcherBuilder::new()
        .with(PlayerControlSystem, "player_control", &[])
        .with(MovementSystem, "movement", &["player_control"])
        .with(
            HardBoundsCheck { padding: 50.0 },
            "hard_bounds_check",
            &["movement"],
        )
        .with(SoftBoundsCheck, "soft_bounds_check", &["movement"])
        .with(
            CollisionDetection,
            "collision_detection",
            &["hard_bounds_check", "soft_bounds_check"],
        )
        .with(
            CollisionHandler,
            "collision_handler",
            &["collision_detection"],
        )
        .with(
            Spawner {
                waves: vec![
                    Wave {
                        spawn_fn: Box::new(|lu, e| spawn_chode((-5.0, 30.0), (2.0, 0.0), lu, e)),
                        repeats: 5,
                        delay: 1.0,
                    },
                    Wave {
                        spawn_fn: Box::new(|lu, e| {
                            spawn_chode((SCREEN_WIDTH + 5.0, 60.0), (-2.0, 0.0), lu, e)
                        }),
                        repeats: 5,
                        delay: 1.0,
                    },
                ],
                state: SpawnerState::Spawning {
                    repeat: 0,
                    wave: 0,
                    cooldown: Default::default(),
                },
            },
            "spawner",
            &["collision_handler"],
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
