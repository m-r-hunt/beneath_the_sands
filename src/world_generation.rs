use crate::prelude::*;

pub struct Dungeon {
    pub position: Vector,
    pub reward: bool,
}

impl Component for Dungeon {
    type Storage = HashMapStorage<Self>;
}

const RADIUS1: f32 = 200.0;
const RADIUS2: f32 = 400.0;

const L1_DUNGEONS: usize = 5;
const L2_DUNGEONS: usize = 10;

pub fn generate_dungeons(world: &mut World) {
    let mut rng = rand::thread_rng();
    let mut out = Vec::new();
    for _ in 0..L1_DUNGEONS {
        out.push(Dungeon {
            position: Vector::from_angle(rng.gen_range(0.0, 360.0))
                .with_len(rng.gen_range(50.0, RADIUS1)),
            reward: false,
        });
    }
    out[rng.gen_range(0, L1_DUNGEONS)].reward = true;
    for _ in 0..L2_DUNGEONS {
        out.push(Dungeon {
            position: Vector::from_angle(rng.gen_range(0.0, 360.0))
                .with_len(rng.gen_range(RADIUS1, RADIUS2)),
            reward: false,
        });
    }
    out[rng.gen_range(L1_DUNGEONS, L2_DUNGEONS)].reward = true;

    for d in out {
        world.create_entity().with(d).build();
    }
}
