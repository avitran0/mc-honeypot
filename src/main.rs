use std::{
    io::Write,
    net::TcpListener,
    sync::{Arc, Mutex},
    thread,
};

use log::{error, info, warn};
use protocol::{mc_version, read, read_header, read_string, read_varint, send_login, send_status};

mod protocol;

fn main() {
    env_logger::builder()
        .format(|buf, record| writeln!(buf, "[{}] {}", record.level(), record.args()))
        .filter_level(log::LevelFilter::Info)
        .init();

    const PORT: u16 = 25565;
    let listener = TcpListener::bind(("0.0.0.0", PORT)).expect("port 25565 is already in use");
    info!("listening on port {PORT}");

    let csv_writer = csv::Writer::from_path("out.csv").unwrap();
    let out_file = Arc::new(Mutex::new(csv_writer));

    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(stream) => stream,
            Err(err) => {
                error!("stream error: {}", err);
                continue;
            }
        };
        let ip = match stream.peer_addr() {
            Ok(ip) => ip,
            Err(_) => {
                error!("could not get connected ip address");
                continue;
            }
        };
        info!("accepted connection from {ip}");

        let thread_file = out_file.clone();

        thread::spawn(move || {
            let header = read_header(&mut stream);
            if header.id != 0x00 {
                warn!("handshake packet id mismatch: 0x{:x}", header.id);
                return;
            }
            let version = read_varint(&mut stream);
            info!("minecraft version: {}", mc_version(version));
            let hostname = read_string(&mut stream);
            info!("hostname: {hostname}");
            let port: u16 = read(&mut stream);
            info!("port: {port}");
            let state: u8 = read(&mut stream);
            let info = match state {
                1 => {
                    info!("type: ping");
                    send_status(&mut stream, version)
                }
                2 => {
                    info!("type: login");
                    send_login(&mut stream)
                }
                _ => None,
            };
            if let Some(mut info) = info {
                info.ip = ip.to_string();
                info.protocol = version;
                info.mc_version = mc_version(version).to_string();
                info.hostname = hostname;
                info.port = port;

                // save info
                let mut writer = thread_file.lock().unwrap();
                writer.serialize(info).unwrap();
                writer.flush().unwrap();
            }

            info!("connection closed\n");
        });
    }
}
