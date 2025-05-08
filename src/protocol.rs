use std::io::{Read, Write};

use bytemuck::AnyBitPattern;
use log::{info, warn};
use serde::Serialize;
use uuid::Uuid;

pub struct PacketHeader {
    pub _length: i32,
    pub id: i32,
}

pub fn read_varint<R: Read>(r: &mut R) -> i32 {
    let mut num_read = 0;
    let mut result = 0;
    loop {
        let mut byte = [0u8];
        if r.read_exact(&mut byte).is_err() {
            return 0;
        }
        let val = (byte[0] & 0x7F) as i32;
        result |= val << (7 * num_read);
        num_read += 1;
        if num_read > 5 {
            eprintln!("varint was too long");
            return 0;
        }
        if (byte[0] & 0x80) == 0 {
            break;
        }
    }
    result
}

fn write_varint<W: Write>(w: &mut W, value: i32) {
    let mut value = value;
    loop {
        if (value & !0x7F) == 0 {
            if w.write_all(&[value as u8]).is_err() {
                warn!("could not write");
            }
            return;
        } else {
            if w.write_all(&[((value & 0x7F) | 0x80) as u8]).is_err() {
                warn!("could not write");
            }
            value = ((value as u32) >> 7) as i32;
        }
    }
}

pub fn read_string<R: Read>(r: &mut R) -> String {
    let length = read_varint(r);
    let mut num_read = 0;
    let mut result = String::with_capacity(16);
    for _ in 0..length {
        let mut byte = [0u8];
        if r.read_exact(&mut byte).is_err() {
            break;
        }
        let val = byte[0] as char;
        result.push(val);
        num_read += 1;
        if num_read > 255 {
            eprintln!("string was too long");
            break;
        }
    }
    result
}

fn read_legacy_string<R: Read>(r: &mut R) -> String {
    let length: i16 = read(r);
    let mut num_read = 0;
    let mut result = String::with_capacity(16);
    for _ in 0..length {
        let mut byte = [0u8];
        if r.read_exact(&mut byte).is_err() {
            break;
        }
        let val = byte[0] as char;
        result.push(val);
        num_read += 1;
        if num_read > 255 {
            eprintln!("string was too long");
            break;
        }
    }
    result
}

fn write_string<W: Write>(w: &mut W, s: &str) {
    write_varint(w, s.len() as i32);
    if w.write_all(s.as_bytes()).is_err() {
        warn!("could not write string");
    }
}

pub fn read<R: Read, T: AnyBitPattern + Default>(r: &mut R) -> T {
    let mut buffer = vec![0u8; std::mem::size_of::<T>()];
    if r.read_exact(&mut buffer).is_err() {
        return T::default();
    };
    // network bytes are in big endian
    #[cfg(target_endian = "little")]
    buffer.reverse();

    bytemuck::try_from_bytes(&buffer)
        .copied()
        .unwrap_or_default()
}

pub fn read_header<R: Read>(r: &mut R) -> PacketHeader {
    PacketHeader {
        _length: read_varint(r),
        id: read_varint(r),
    }
}

#[derive(Debug, Serialize, Default)]
pub struct LoginInfo {
    pub ip: String,
    pub protocol: i32,
    pub mc_version: String,
    pub hostname: String,
    pub port: u16,
    pub player_name: String,
    pub uuid: String,
}

#[derive(Serialize)]
struct Status {
    pub version: StatusVersion,
    pub players: StatusPlayers,
    pub description: StatusDescription,
}

impl Status {
    pub fn new(protocol: i32, description: String) -> Self {
        Self {
            version: StatusVersion {
                name: mc_version(protocol).to_string(),
                protocol,
            },
            players: StatusPlayers {
                max: 100,
                online: 0,
                sample: Vec::new(),
            },
            description: StatusDescription { text: description },
        }
    }
}

#[derive(Serialize)]
struct StatusVersion {
    pub name: String,
    pub protocol: i32,
}

#[derive(Serialize)]
struct StatusPlayers {
    pub max: i32,
    pub online: i32,
    pub sample: Vec<()>,
}

#[derive(Serialize)]
struct StatusDescription {
    pub text: String,
}

pub fn send_status<S: Read + Write>(s: &mut S, protocol: i32) -> Option<LoginInfo> {
    let header = read_header(s);
    if header.id != 0x00 {
        warn!("status packet id mismatch: 0x{:x}", header.id);
        return None;
    }
    let protocol = if protocol < 0 { 770 } else { protocol };
    let server_description = Status::new(protocol, "A Minecraft Server".to_string());
    let mut payload = Vec::new();
    write_varint(&mut payload, 0x00);
    write_string(
        &mut payload,
        &serde_json::to_string(&server_description).unwrap(),
    );

    // write length and send
    write_varint(s, payload.len() as i32);
    s.write_all(&payload).unwrap();

    // ping packet
    let header = read_header(s);
    if header.id != 0x01 {
        warn!("ping packet id mismatch: 0x{:x}", header.id);
        return None;
    }
    let mut ping_payload = [0u8; 8];
    s.read_exact(&mut ping_payload).unwrap();

    // echo back ping payload as pong
    let mut pong = Vec::new();
    write_varint(&mut pong, 0x01);
    pong.extend_from_slice(&ping_payload);

    write_varint(s, pong.len() as i32);
    s.write_all(&pong).unwrap();
    None
}

pub fn send_legacy_ping<S: Read + Write>(s: &mut S) {
    let _id: u8 = read(s);
    let _ping_payload: u8 = read(s);
    let _plugin_id: u8 = read(s);

    let hostname = read_legacy_string(s);
    let _remaining_length: i16 = read(s);
    let version: u8 = read(s);
    let _client_name = read_legacy_string(s);
    let port: i32 = read(s);
    info!("protocol version: {version}");
    info!("minecraft version: {}", mc_version_legacy(version));
    info!("hostname: {hostname}");
    info!("port: {port}");

    let payload = [
        0xFF, // packet id
        0x00, 0x23, // length of packet
        0x00, 0xA7, 0x00, 0x31, 0x00, 0x00, // magic prefix
        0x00, 0x37, 0x00, 0x38, 0x00, 0x00, // protocol version 78
        0x00, 0x31, 0x00, 0x2E, 0x00, 0x36, 0x00, 0x2E, 0x00, 0x34, 0x00,
        0x00, // version 1.6.4
        0x00, 0x41, 0x00, 0x20, 0x00, 0x4D, 0x00, 0x69, 0x00, 0x6E, 0x00, 0x65, 0x00, 0x63, 0x00,
        0x72, 0x00, 0x61, 0x00, 0x66, 0x00, 0x74, 0x00, 0x20, 0x00, 0x53, 0x00, 0x65, 0x00, 0x72,
        0x00, 0x76, 0x00, 0x65, 0x00, 0x72, 0x00, 0x00, // "A Minecraft Server"
        0x00, 0x30, 0x00, 0x00, // 0 current players
        0x00, 0x32, 0x00, 0x30, // 20 max players
    ];

    s.write_all(&payload).unwrap();
}

pub fn send_login<S: Read + Write>(s: &mut S) -> Option<LoginInfo> {
    let login_start_header = read_header(s);
    if login_start_header.id != 0 {
        warn!(
            "login start packet id mismatch: 0x{:x}",
            login_start_header.id
        );
        return None;
    }

    let player_name = read_string(s);
    info!("player name: {player_name}");
    let uuid = Uuid::from_u128_le(read(s));
    info!("player uuid: {uuid}");

    Some(LoginInfo {
        player_name,
        uuid: uuid.to_string(),
        ..Default::default()
    })
}

pub const fn mc_version(protocol_version: i32) -> &'static str {
    match protocol_version {
        770 => "1.21.5",
        769 => "1.21.4",
        768 => "1.21.3",
        767 => "1.21.1",
        766 => "1.20.6",
        765 => "1.20.4",
        764 => "1.20.2",
        763 => "1.20.1",
        762 => "1.19.4",
        761 => "1.19.3",
        760 => "1.19.2",
        759 => "1.19",
        758 => "1.18.2",
        757 => "1.18.1",
        756 => "1.17.1",
        755 => "1.17",
        754 => "1.16.5",
        753 => "1.16.3",
        751 => "1.16.2",
        736 => "1.16.1",
        735 => "1.16",
        578 => "1.15.2",
        575 => "1.15.1",
        573 => "1.15",
        498 => "1.14.4",
        490 => "1.14.3",
        485 => "1.14.2",
        480 => "1.14.1",
        477 => "1.14",
        404 => "1.13.2",
        401 => "1.13.1",
        393 => "1.13",
        340 => "1.12.2",
        338 => "1.12.1",
        335 => "1.12",
        316 => "1.11.2",
        315 => "1.11",
        210 => "1.10.2",
        110 => "1.9.4",
        109 => "1.9.2",
        108 => "1.9.1",
        107 => "1.9",
        47 => "1.8.9",
        5 => "1.7.10",
        4 => "1.7.5",
        3 => "1.7",
        _ => "unknown",
    }
}

pub const fn mc_version_legacy(protocol_version: u8) -> &'static str {
    match protocol_version {
        78 => "1.6.4",
        74 => "1.6.2",
        73 => "1.6.1",
        61 => "1.5.2",
        60 => "1.5.1",
        51 => "1.4.7",
        49 => "1.4.5",
        47 => "1.4.2",
        39 => "1.3.2",
        29 => "1.2.5",
        28 => "1.2.3",
        23 => "1.1",
        22 => "1.0.1",
        21 => "b1.9-pre5",
        20 => "b1.9-pre4",
        19 => "b1.9-pre3",
        18 => "b1.9-pre1",
        17 => "b1.8.1",
        14 => "b1.7.3",
        13 => "b1.6.6",
        11 => "b1.5",
        10 => "b1.4",
        9 => "b1.3",
        8 => "b1.2",
        7 => "b1.1",
        _ => "unknown",
    }
}
