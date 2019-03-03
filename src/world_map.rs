use crate::prelude::*;
use crate::world_generation::Dungeon;
use crate::{Input, ScreenSize, UIState};

pub struct WorldMapScreen;

impl<'a> System<'a> for WorldMapScreen {
    type SystemData = (
        ReadStorage<'a, Dungeon>,
        Write<'a, UIState>,
        Read<'a, ScreenSize>,
        Read<'a, Input>,
    );

    fn run(&mut self, (dungeons, mut ui_state, screen_size, input): Self::SystemData) {
        let offset = screen_size.size / 2.0;
        let mouse_pos = input.raw_mouse_pos - offset;
        for d in dungeons.join() {
            if input.fire && (d.position - mouse_pos).len2() < 10.0 * 10.0 {
                *ui_state = UIState::Playing;
            }
        }
    }
}
