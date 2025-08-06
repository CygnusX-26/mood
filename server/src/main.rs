use std::{io, net::Ipv4Addr};

use log::info;
use server::Server;

fn main() -> io::Result<()> {
    env_logger::init();
    let ip = Ipv4Addr::new(0, 0, 0, 0);
    let port = 8003;
    let mut server = Server::new(ip, port, 20)?;
    info!("starting server at {ip}:{port}");
    server.run();
    Ok(())
}
