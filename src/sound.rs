use crate::prelude::*;
use quicksilver::lifecycle::Asset;
use quicksilver::sound::Sound;
use std::collections::HashMap;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum SoundRequest {
    PlayerShot,
    PlayerDeath,
    EnemyShot,
    EnemyDeath,
    BossShot,
    BossDeath,
    Pause,
}

#[derive(Debug, Default, Clone)]
pub struct SoundQueue {
    events: Vec<SoundRequest>,
}

impl SoundQueue {
    fn clear(&mut self) {
        self.events.clear();
    }

    pub fn enqueue(&mut self, event: SoundRequest) {
        self.events.push(event);
    }

    fn iter(&self) -> impl Iterator<Item = &SoundRequest> {
        self.events.iter()
    }
}

pub struct SoundSystem {
    sounds: HashMap<SoundRequest, Asset<Sound>>,
}

impl SoundSystem {
    pub fn new() -> Self {
        let mut sounds = HashMap::new();
        sounds.insert(
            SoundRequest::PlayerShot,
            Asset::new(Sound::load("oryx_8-bit_sounds/abilities/shoot_a.wav")),
        );
        sounds.insert(
            SoundRequest::PlayerDeath,
            Asset::new(Sound::load("oryx_8-bit_sounds/interface/lose_b.wav")),
        );
        sounds.insert(
            SoundRequest::EnemyShot,
            Asset::new(Sound::load("oryx_8-bit_sounds/abilities/shoot_b.wav")),
        );
        sounds.insert(
            SoundRequest::EnemyDeath,
            Asset::new(Sound::load("oryx_8-bit_sounds/impacts/impact_b.wav")),
        );
        sounds.insert(
            SoundRequest::BossShot,
            Asset::new(Sound::load("oryx_8-bit_sounds/abilities/summon.wav")),
        );
        sounds.insert(
            SoundRequest::BossDeath,
            Asset::new(Sound::load("oryx_8-bit_sounds/impacts/boom_a.wav")),
        );
        sounds.insert(
            SoundRequest::Pause,
            Asset::new(Sound::load("oryx_8-bit_sounds/interface/end_turn.wav")),
        );
        SoundSystem { sounds }
    }
}

impl<'a> System<'a> for SoundSystem {
    type SystemData = Write<'a, SoundQueue>;

    fn run(&mut self, mut sound_queue: Self::SystemData) {
        for sound in sound_queue.iter() {
            self.sounds
                .get_mut(&sound)
                .unwrap()
                .execute(|s| s.play())
                .unwrap();
        }
        sound_queue.clear();
    }
}
