use crate::level_generation::LevelStyle;
use crate::prelude::*;
use crate::world_map::{self, Dungeon, Item, Reward, RANGE1, RANGE2};

const L1_DUNGEONS: usize = 2;
const L2_DUNGEONS: usize = 2;

pub fn generate_dungeons(world: &mut World) {
    let mut rng = rand::thread_rng();
    let mut out = Vec::new();
    let _items = world_map::all_items();
    for _ in 0..L1_DUNGEONS {
        out.push(Dungeon {
            position: Vector::from_angle(rng.gen_range(0.0, 360.0))
                .with_len(rng.gen_range(50.0, RANGE1)),
            reward: Reward::Choice(Item::SpeedIncrease, Item::Backfire),
            completed: false,
            style: if rng.gen_range(0.0, 1.0) > 0.5 {
                LevelStyle::Cyclic
            } else {
                LevelStyle::CellularAutomata
            },
        });
    }
    out.push(Dungeon {
        position: Vector::from_angle(rng.gen_range(0.0, 360.0))
            .with_len(rng.gen_range(50.0, RANGE1)),
        reward: Reward::Progress,
        completed: false,
        style: if rng.gen_range(0.0, 1.0) > 0.5 {
            LevelStyle::Cyclic
        } else {
            LevelStyle::CellularAutomata
        },
    });

    for _ in 0..L2_DUNGEONS {
        out.push(Dungeon {
            position: Vector::from_angle(rng.gen_range(0.0, 360.0))
                .with_len(rng.gen_range(RANGE1, RANGE2)),
            reward: Reward::Choice(Item::AttackSpeed, Item::MaxHealth),
            completed: false,
            style: if rng.gen_range(0.0, 1.0) > 0.5 {
                LevelStyle::Cyclic
            } else {
                LevelStyle::CellularAutomata
            },
        });
    }
    out.push(Dungeon {
        position: Vector::from_angle(rng.gen_range(0.0, 360.0))
            .with_len(rng.gen_range(RANGE1, RANGE2)),
        reward: Reward::Progress,
        completed: false,
        style: if rng.gen_range(0.0, 1.0) > 0.5 {
            LevelStyle::Cyclic
        } else {
            LevelStyle::CellularAutomata
        },
    });

    for d in out {
        world.create_entity().with(d).build();
    }
}
