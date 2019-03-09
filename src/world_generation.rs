use crate::level_generation::LevelStyle;
use crate::prelude::*;
use crate::world_map::{self, Dungeon, Reward, RANGE1, RANGE2};

const L1_DUNGEONS: usize = 2;
const L2_DUNGEONS: usize = 2;

pub fn generate_dungeons(world: &mut World) {
    let mut rng = rand::thread_rng();
    let mut out = Vec::new();
    let mut items = world_map::all_items();
    for _ in 0..L1_DUNGEONS {
        let index1 = rng.gen_range(0, items.len());
        let item1 = items[index1];
        items.remove(index1);
        let index2 = rng.gen_range(0, items.len());
        let item2 = items[index2];
        items.remove(index2);
        out.push(Dungeon {
            position: Vector::from_angle(rng.gen_range(0.0, 360.0))
                .with_len(rng.gen_range(50.0, RANGE1)),
            reward: Reward::Choice(item1, item2),
            completed: false,
            style: if rng.gen_range(0.0, 1.0) > 0.5 {
                LevelStyle::Cyclic
            } else {
                LevelStyle::CellularAutomata
            },
            difficulty: 1,
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
        difficulty: 1,
    });

    for _ in 0..L2_DUNGEONS {
        let index1 = rng.gen_range(0, items.len());
        let item1 = items[index1];
        items.remove(index1);
        let index2 = rng.gen_range(0, items.len());
        let item2 = items[index2];
        items.remove(index2);
        out.push(Dungeon {
            position: Vector::from_angle(rng.gen_range(0.0, 360.0))
                .with_len(rng.gen_range(RANGE1, RANGE2)),
            reward: Reward::Choice(item1, item2),
            completed: false,
            style: if rng.gen_range(0.0, 1.0) > 0.5 {
                LevelStyle::Cyclic
            } else {
                LevelStyle::CellularAutomata
            },
            difficulty: 2,
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
        difficulty: 2,
    });

    for d in out {
        world.create_entity().with(d).build();
    }
}
