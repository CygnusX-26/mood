use std::error::Error;
use std::rc::Rc;

use log::Level;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::player_state::PlayerState;
use bincode::config as bconfig;
use bincode::{config::Configuration, serde as bserde};

#[derive(Serialize, Deserialize)]
pub struct Command {
    pub command_type: CommandType,
    pub time: u128,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CommandType {
    PlayerJoin,
    PlayerLeave,
    PlayerMove {
        position: [f32; 3],
        velocity: [f32; 3],
        pitch: f32,
        yaw: f32,
        body_yaw: f32,
    },
    Data((Uuid, Rc<[PlayerState]>)),
}

impl Command {
    const CONFIG: Configuration = bconfig::standard();

    pub fn serialize(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        Ok(bserde::encode_to_vec(self, Self::CONFIG)?)
    }

    pub fn deserialize(data: &[u8]) -> Result<Self, Box<dyn Error>> {
        let command: (Command, _) = bserde::decode_from_slice(data, Self::CONFIG)?;
        Ok(command.0)
    }
}

impl CommandType {
    pub fn log_level(&self) -> Level {
        match self {
            Self::PlayerMove { .. } => Level::Debug,
            _ => Level::Info,
        }
    }
}
