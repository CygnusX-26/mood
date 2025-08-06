use std::time::Instant;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PlayerState {
    pub player_id: Uuid,
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub pitch: f32,
    pub yaw: f32,
    pub body_yaw: f32,
    pub health: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct TimedPlayerState {
    pub player_state: PlayerState,
    pub time: Instant,
}

impl TimedPlayerState {
    pub fn new(player_state: PlayerState) -> Self {
        Self {
            player_state,
            time: Instant::now(),
        }
    }
}
