use std::{
    collections::{HashMap, VecDeque},
    io,
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    rc::Rc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use command::{Command, CommandType};
use game::player_state::PlayerState;
use log::{error, info, log, warn};

mod command;
mod game;

pub struct Server {
    socket: UdpSocket,
    input_commands: VecDeque<InputCommand>,
    player_states: HashMap<SocketAddr, PlayerState>,
    last_packet_sent: HashMap<SocketAddr, Instant>,
    last_tick: Instant,
    tick_rate: Duration,
    ticks_elapsed: u64,
}

struct InputCommand {
    command: Command,
    src_addr: SocketAddr,
}
impl Server {
    const MAX_PLAYERS: u8 = 32;
    pub fn new(server_addr: Ipv4Addr, port: u16, tick_rate_in_millis: u64) -> io::Result<Self> {
        let addr = SocketAddr::new(IpAddr::V4(server_addr), port);
        let socket = UdpSocket::bind(addr)?;
        socket.set_nonblocking(true)?;

        Ok(Self {
            socket,
            input_commands: VecDeque::new(),
            player_states: HashMap::new(),
            last_packet_sent: HashMap::new(),
            last_tick: Instant::now(),
            tick_rate: Duration::from_millis(tick_rate_in_millis),
            ticks_elapsed: 0,
        })
    }

    pub fn run(&mut self) {
        let mut buffer = [0; 1024];
        loop {
            self.poll_connections(&mut buffer);

            let now = Instant::now();
            if now.duration_since(self.last_tick) >= self.tick_rate {
                self.process_game_tick();
                self.last_tick = now;
                self.ticks_elapsed += 1;
                self.input_commands.clear();
                self.cull_dead_connections();
                self.emit_game_state();
            }
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    fn poll_connections(&mut self, buffer: &mut [u8]) {
        match self.socket.recv_from(buffer) {
            Ok((number_of_bytes, src_addr)) => {
                if let Ok(command) = Command::deserialize(&buffer[..number_of_bytes]) {
                    log!(
                        command.command_type.log_level(),
                        "{} sent {:?}",
                        src_addr,
                        command.command_type
                    );
                    self.input_commands
                        .push_back(InputCommand { command, src_addr });
                } else {
                    warn!("{src_addr} sent an invalid command");
                }
            }
            Err(e) if e.kind() != io::ErrorKind::WouldBlock => {
                error!("{e}");
            }
            _ => {}
        }
    }

    fn process_game_tick(&mut self) {
        loop {
            let Some(input_command) = self.input_commands.pop_front() else {
                break;
            };

            let command = input_command.command;
            let src_addr = input_command.src_addr;

            match command.command_type {
                CommandType::PlayerJoin => {
                    if (self.player_states.len() as u8) < Self::MAX_PLAYERS {
                        self.player_states.entry(src_addr).or_default();
                        self.last_packet_sent.insert(src_addr, Instant::now());
                    }
                }
                CommandType::PlayerLeave => {
                    self.player_states.remove(&src_addr);
                    self.last_packet_sent.remove(&src_addr);
                }
                CommandType::PlayerMove {
                    position,
                    velocity,
                    pitch,
                    yaw,
                    body_yaw,
                } => {
                    self.last_packet_sent.insert(src_addr, Instant::now());
                    if let Some(player) = self.player_states.get_mut(&src_addr) {
                        player.update(position, velocity, pitch, yaw, body_yaw);
                    }
                }
                _ => {}
            }
        }
    }

    fn emit_game_state(&self) {
        let collected_states: Rc<[PlayerState]> = Rc::from(
            self.player_states
                .values()
                .cloned()
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        );
        self.player_states.iter().for_each(|(src_addr, state)| {
            let game_state = Command {
                command_type: CommandType::Data((state.player_id, collected_states.clone())),
                time: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis(),
            }
            .serialize();
            if let Ok(serialized_state) = game_state {
                let Ok(_) = self.socket.send_to(&serialized_state, src_addr) else {
                    error!("failed to send data to {src_addr}");
                    return;
                };
            }
        });
    }

    pub fn cull_dead_connections(&mut self) {
        let addresses_to_remove: Vec<SocketAddr> = self
            .last_packet_sent
            .iter()
            .filter(|(_, time)| time.elapsed() > Duration::from_secs(5))
            .map(|(&src_addr, _)| src_addr)
            .collect();

        for addr in &addresses_to_remove {
            info!("Culling connection from {addr}");
            self.last_packet_sent.remove(addr);
            self.player_states.remove(addr);
        }

        if !addresses_to_remove.is_empty() {
            info!("Current connections: {:?}", self.player_states.keys());
        }
    }
}
