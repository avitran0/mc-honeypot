use std::{io::Write, net::TcpListener, thread, time::Instant};

use log::{error, info, warn};
use protocol::{mc_version, read, read_header, read_string, read_varint, send_login, send_status};

mod protocol;

fn main() {
    env_logger::builder()
        .format(|buf, record| writeln!(buf, "[{}] {}", record.level(), record.args()))
        .filter_level(log::LevelFilter::Info)
        .init();

    const PORT: u16 = 25565;
    let listener = TcpListener::bind(("0.0.0.0", PORT)).unwrap();
    info!("listening on port {PORT}");

    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(stream) => stream,
            Err(err) => {
                error!("stream error: {}", err);
                continue;
            }
        };
        info!("accepted connection from {}", stream.peer_addr().unwrap());

        thread::spawn(move || {
            let header = read_header(&mut stream);
            if header.id != 0x00 {
                warn!("packet id mismatch");
                return;
            }
            let version = read_varint(&mut stream);
            info!("minecraft version: {}", mc_version(version));
            let hostname = read_string(&mut stream);
            info!("hostname: {hostname}");
            let port: u16 = read(&mut stream);
            info!("port: {port}");
            let state: u8 = read(&mut stream);
            match state {
                1 => send_status(&mut stream),
                2 => send_login(&mut stream),
                _ => {}
            }

            info!("connection closed");
        });
    }
}
