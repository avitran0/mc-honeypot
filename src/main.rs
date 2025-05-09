use std::{
    io::Write,
    net::TcpListener,
    sync::{Arc, LazyLock, Mutex},
    thread,
};

use clap::Parser;
use formats::{
    LoginEvent, MultiSink, csv::CsvEventSink, json::JsonEventSink, sqlite::SqliteEventSink,
};
use log::{error, info, warn};
use packets::*;

mod formats;
mod packets;
mod protocol;
mod util;

/// a minecraft honeypot
#[derive(Parser)]
#[command(about, version, author = "avitran0")]
struct Args {
    /// what port the server should listen on
    #[arg(short, long, default_value_t = 25565)]
    port: u16,

    /// message of the day
    #[arg(short, long, default_value = "A Minecraft Server")]
    message: String,

    /// max amount of players on the server
    #[arg(long, default_value_t = 20)]
    max_players: i32,

    /// online player count
    #[arg(long, default_value_t = 0)]
    online_players: i32,

    /// output file name
    #[arg(long, default_value = "logins")]
    file_name: String,

    /// comma-separated list of formats (json, sqlite etc...)
    #[arg(short, long, default_value = "json")]
    formats: String,
}

static ARGS: LazyLock<Args> = LazyLock::new(Args::parse);

fn main() {
    println!("{}", ARGS.message);

    env_logger::builder()
        .format(|buf, record| writeln!(buf, "[{}] {}", record.level(), record.args()))
        .filter_level(log::LevelFilter::Info)
        .init();

    // init all file formats
    let mut sink = MultiSink::new();
    let formats = ARGS.formats.to_lowercase();
    for format in formats.split(',') {
        match format {
            "json" => sink.add_sink(JsonEventSink::new()),
            "csv" => sink.add_sink(CsvEventSink::new()),
            "sqlite" => sink.add_sink(SqliteEventSink::new()),
            _ => {
                warn!("invalid format: {format}");
                continue;
            }
        }
    }
    info!("chosen formats: {}", sink.sink_names());

    let shared_sink = Arc::new(Mutex::new(sink));

    let listener = match TcpListener::bind(("0.0.0.0", ARGS.port)) {
        Ok(listener) => listener,
        Err(error) => {
            match error.kind() {
                std::io::ErrorKind::AddrInUse => error!("port {} is already in use", ARGS.port),
                std::io::ErrorKind::PermissionDenied => {
                    error!("missing permissions to bind port {}", ARGS.port)
                }
                _ => {}
            };
            return;
        }
    };
    info!("listening on port {}", ARGS.port);

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
            Err(_) => continue,
        };

        let thread_sink = shared_sink.clone();

        thread::spawn(move || {
            println!();
            let mut first_byte = [0u8; 1];
            stream.peek(&mut first_byte).unwrap();
            // legacy ping
            if first_byte[0] == 0xFE {
                let ping = LegacyPing::new(&mut stream);
                LegacyPingResponse::send(&mut stream);
                info!("legacy ping from {ip}");
                info!("protocol version: {}", ping.version);
                info!("minecraft version: {}", ping.mc_version);
                info!("hostname: {}", ping.hostname);
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
                info!("ping from: {ip}");
                info!("protocol version: {}", handshake.version);
                info!("minecraft version: {}", &handshake.mc_version);
                info!("hostname: {}", &handshake.hostname);
                return;
            }

            if handshake.state != 2 {
                return;
            }

            info!("login from: {ip}");
            info!("protocol version: {}", handshake.version);
            info!("minecraft version: {}", &handshake.mc_version);
            info!("hostname: {}", &handshake.hostname);

            let login = LoginStart::new(&mut stream);
            info!("player name: {}", &login.player_name);
            info!("player uuid: {}", login.uuid);

            thread_sink.lock().unwrap().write(&LoginEvent {
                ip,
                version: handshake.version,
                mc_version: handshake.mc_version,
                hostname: handshake.hostname,
                player_name: login.player_name,
                player_uuid: login.uuid,
                timestamp: chrono::Local::now(),
            });
        });
    }
}
