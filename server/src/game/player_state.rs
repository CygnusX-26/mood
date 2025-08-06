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

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            player_id: Uuid::new_v4(),
            position: [0.0, 0.0, 0.0],
            velocity: [0.0, 0.0, 0.0],
            pitch: 0.0,
            yaw: 0.0,
            body_yaw: 0.0,
            health: 100,
        }
    }
}

impl PlayerState {
    pub fn update(
        &mut self,
        position: [f32; 3],
        velocity: [f32; 3],
        pitch: f32,
        yaw: f32,
        body_yaw: f32,
    ) {
        self.position = position;
        self.velocity = velocity;
        self.pitch = pitch;
        self.yaw = yaw;
        self.body_yaw = body_yaw;
    }
}
