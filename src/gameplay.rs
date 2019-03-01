use super::physics::{HitBox, Movement};
use super::render::RenderComponent;
use super::{Event, EventQueue, SimTime, Timer, WorldBounds};
use quicksilver::graphics::Color;
use specs::prelude::*;

pub struct HardBoundsCheck {
    pub padding: f32,
}

impl<'a> System<'a> for HardBoundsCheck {
    type SystemData = (
        ReadStorage<'a, Movement>,
        Entities<'a>,
        Read<'a, WorldBounds>,
    );

    fn run(&mut self, (movement, entities, bounds): Self::SystemData) {
        for (movement, entity) in (&movement, &*entities).join() {
            if movement.position.0 < bounds.left - self.padding
                || movement.position.0 > bounds.right + self.padding
                || movement.position.1 < bounds.top - self.padding
                || movement.position.1 > bounds.bottom + self.padding
            {
                entities.delete(entity).unwrap();
            }
        }
    }
}

pub struct CollisionHandler;

impl<'a> System<'a> for CollisionHandler {
    type SystemData = (Read<'a, EventQueue>, Entities<'a>);

    fn run(&mut self, (event_queue, entities): Self::SystemData) {
        for event in event_queue.iter() {
            match event {
                Event::Collision(hit, hit_by) => {
                    entities.delete(*hit).unwrap();
                    entities.delete(*hit_by).unwrap();
                }
            }
        }
    }
}

#[derive(Default)]
pub struct Spawned;

impl Component for Spawned {
    type Storage = NullStorage<Self>;
}

pub struct Spawner {
    pub waves: Vec<Wave>,
    pub state: SpawnerState,
}

#[derive(Debug, Copy, Clone)]
pub enum SpawnerState {
    Spawning {
        wave: usize,
        repeat: usize,
        cooldown: Timer,
    },
    Done,
}

enum SpawnAction {
    None,
    Spawn(usize),
}

impl SpawnerState {
    fn run(
        &self,
        sim_time: SimTime,
        waves: &[Wave],
        num_spawned: usize,
    ) -> (SpawnAction, SpawnerState) {
        match self {
            SpawnerState::Spawning {
                repeat,
                wave,
                cooldown,
            } if cooldown.expired(sim_time) && *repeat < waves[*wave].repeats => (
                SpawnAction::Spawn(*wave),
                SpawnerState::Spawning {
                    repeat: repeat + 1,
                    wave: *wave,
                    cooldown: Timer::new_set(sim_time, waves[*wave].delay),
                },
            ),
            SpawnerState::Spawning { repeat, wave, .. }
                if num_spawned == 0
                    && *repeat >= waves[*wave].repeats
                    && wave + 1 < waves.len() =>
            {
                (
                    SpawnAction::None,
                    SpawnerState::Spawning {
                        repeat: 0,
                        wave: wave + 1,
                        cooldown: Default::default(),
                    },
                )
            }
            SpawnerState::Spawning { repeat, wave, .. }
                if num_spawned == 0 && *repeat >= waves[*wave].repeats =>
            {
                (SpawnAction::None, SpawnerState::Done)
            }
            _ => (SpawnAction::None, *self),
        }
    }
}

pub struct Wave {
    pub spawn_fn: Box<Fn(&LazyUpdate, &Entities) + Send>,
    pub repeats: usize,
    pub delay: f32,
}

impl<'a> System<'a> for Spawner {
    type SystemData = (
        Read<'a, LazyUpdate>,
        Entities<'a>,
        ReadStorage<'a, Spawned>,
        Read<'a, SimTime>,
    );

    fn run(&mut self, (lazy_update, entities, spawned, sim_time): Self::SystemData) {
        let (action, new_state) = self
            .state
            .run(*sim_time, &self.waves, spawned.join().count());
        match action {
            SpawnAction::Spawn(w) => (self.waves[w].spawn_fn)(&lazy_update, &entities),
            SpawnAction::None => (),
        }
        self.state = new_state;
    }
}

pub fn spawn_chode(
    coords: (f32, f32),
    velocity: (f32, f32),
    lazy_update: &LazyUpdate,
    entities: &Entities,
) {
    lazy_update
        .create_entity(entities)
        .with(Movement {
            position: coords,
            velocity,
        })
        .with(Spawned)
        .with(RenderComponent {
            width: 20.0,
            height: 20.0,
            colour: Color::RED,
        })
        .with(HitBox {
            width: 20.0,
            height: 20.0,
        })
        .build();
}
