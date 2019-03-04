use crate::prelude::*;

pub struct Dungeon {
    pub position: Vector,
    pub reward: bool,
    pub completed: bool,
}

impl Component for Dungeon {
    type Storage = HashMapStorage<Self>;
}

pub const RANGE1: f32 = 150.0;
pub const RANGE2: f32 = 300.0;

const L1_DUNGEONS: usize = 5;
const L2_DUNGEONS: usize = 10;

pub fn generate_dungeons(world: &mut World) {
    let mut rng = rand::thread_rng();
    let mut out = Vec::new();
    for _ in 0..L1_DUNGEONS {
        out.push(Dungeon {
            position: Vector::from_angle(rng.gen_range(0.0, 360.0))
                .with_len(rng.gen_range(50.0, RANGE1)),
            reward: false,
            completed: false,
        });
    }
    out[rng.gen_range(0, L1_DUNGEONS)].reward = true;
    for _ in 0..L2_DUNGEONS {
        out.push(Dungeon {
            position: Vector::from_angle(rng.gen_range(0.0, 360.0))
                .with_len(rng.gen_range(RANGE1, RANGE2)),
            reward: false,
            completed: false,
        });
    }
    out[rng.gen_range(L1_DUNGEONS, L2_DUNGEONS)].reward = true;

    for d in out {
        world.create_entity().with(d).build();
    }
}
