use std::{io::Write, net::TcpListener, thread};

use log::{error, info};
use packets::{
    Handshake, LegacyPing, LegacyPingResponse, LoginStart, LoginSuccess, Ping, Pong, StatusRequest,
    StatusResponse,
};

mod packets;
mod protocol;
mod util;

fn main() {
    env_logger::builder()
        .format(|buf, record| writeln!(buf, "[{}] {}", record.level(), record.args()))
        .filter_level(log::LevelFilter::Info)
        .init();

    const PORT: u16 = 25565;
    let listener = TcpListener::bind(("0.0.0.0", PORT)).expect("port 25565 is already in use");
    info!("listening on port {PORT}");

    for stream in listener.incoming() {
        println!();
        let mut stream = match stream {
            Ok(stream) => stream,
            Err(err) => {
                error!("stream error: {}", err);
                continue;
            }
        };
        let ip = match stream.peer_addr() {
            Ok(ip) => ip,
            Err(_) => continue,
        };

        thread::spawn(move || {
            let mut first_byte = [0u8; 1];
            stream.peek(&mut first_byte).unwrap();
            // legacy ping
            if first_byte[0] == 0xFE {
                let _ping = LegacyPing::new(&mut stream);
                LegacyPingResponse::send(&mut stream);
                return;
            }

            // normal ping or handshake
            let handshake = Handshake::new(&mut stream);

            // modern ping
            if handshake.state == 1 {
                let _status_request = StatusRequest::new(&mut stream);
                StatusResponse::send(&mut stream, handshake.version);
                let ping = Ping::new(&mut stream);
                Pong::send(&mut stream, ping.payload);
                return;
            }

            if handshake.state != 2 {
                return;
            }

            info!("ip: {ip}");
            info!("protocol version: {}", handshake.version);
            info!("minecraft version: {}", &handshake.mc_version);
            info!("hostname: {}", &handshake.hostname);
            info!("port: {}", handshake.port);

            let login = LoginStart::new(&mut stream);
            LoginSuccess::send(&mut stream, &login);
        });
    }
}
