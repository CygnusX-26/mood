use std::{
    collections::HashMap,
    io,
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    time::{SystemTime, UNIX_EPOCH},
};

use command::{Command, CommandType};
use log::{error, info};
use player_state::TimedPlayerState;
use uuid::Uuid;

pub mod command;
pub mod player_state;

pub struct Network {
    socket: UdpSocket,
    pub player_states: HashMap<Uuid, TimedPlayerState>,
}

impl Network {
    pub fn new(ip_addr: Ipv4Addr, port: u16) -> io::Result<Self> {
        let addr = SocketAddr::new(IpAddr::V4(ip_addr), port);
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.connect(addr)?;
        socket.set_nonblocking(true)?;

        Ok(Self {
            socket,
            player_states: HashMap::new(),
        })
    }

    pub fn poll(&mut self) {
        let mut buffer = [0; 1024];
        match self.socket.recv_from(&mut buffer) {
            Ok((number_of_bytes, src_addr)) => {
                if let Ok(command) = Command::deserialize(&buffer[..number_of_bytes]) {
                    self.player_states.clear();
                    info!("recieved {:?} from {}", command.command_type, src_addr);
                    self.handle_command(command.command_type);
                }
            }
            Err(e) if e.kind() != io::ErrorKind::WouldBlock => {
                error!("{e}");
            }
            _ => {}
        }
    }

    pub fn send_player_join(&self) -> io::Result<()> {
        let connect_command = Command {
            command_type: command::CommandType::PlayerJoin,
            time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis(),
        };
        self.socket.send(&connect_command.serialize().unwrap())?;
        Ok(())
    }

    pub fn send_player_leave(&self) -> io::Result<()> {
        let disconnect_command = Command {
            command_type: command::CommandType::PlayerLeave,
            time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis(),
        };
        self.socket.send(&disconnect_command.serialize().unwrap())?;
        Ok(())
    }

    pub fn send_player_move(
        &self,
        position: [f32; 3],
        velocity: [f32; 3],
        pitch: f32,
        yaw: f32,
        body_yaw: f32,
    ) -> io::Result<()> {
        let movement_command = Command {
            command_type: command::CommandType::PlayerMove {
                position,
                velocity,
                pitch,
                yaw,
                body_yaw,
            },
            time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis(),
        };
        self.socket.send(&movement_command.serialize().unwrap())?;
        Ok(())
    }

    fn handle_command(&mut self, command: CommandType) {
        if let CommandType::Data((uuid, player_states)) = command {
            player_states.as_ref().iter().for_each(|player_state| {
                let player_id = player_state.player_id;
                if player_id != uuid {
                    self.player_states
                        .insert(player_state.player_id, TimedPlayerState::new(*player_state));
                }
            });
        }
    }
}
