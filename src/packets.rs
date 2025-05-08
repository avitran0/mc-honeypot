use std::io::{Read, Write};

use serde::Serialize;
use uuid::Uuid;

use crate::{
    protocol::{PacketHeader, mc_version, mc_version_legacy, read_header},
    util::{read, read_legacy_string, read_string, read_varint, write_string, write_varint},
};

/// first packet sent by modern clients
pub struct Handshake {
    pub _header: PacketHeader,
    pub version: i32,
    pub mc_version: String,
    pub hostname: String,
    pub port: u16,
    pub state: i32,
}

impl Handshake {
    pub fn new<R: Read>(r: &mut R) -> Self {
        let _header = read_header(r);
        let version = read_varint(r);
        let hostname = read_string(r);
        let port = read(r);
        let state = read_varint(r);
        Self {
            _header,
            version,
            mc_version: mc_version(version).to_string(),
            hostname,
            port,
            state,
        }
    }
}

/// requests server description
pub struct StatusRequest {
    pub _header: PacketHeader,
}

impl StatusRequest {
    pub fn new<R: Read>(r: &mut R) -> Self {
        let _header = read_header(r);
        Self { _header }
    }
}

/// Status Response sends server description
pub struct StatusResponse;

impl StatusResponse {
    pub fn send<W: Write>(w: &mut W, version: i32) {
        let protocol = if version < 0 { 770 } else { version };
        let server_description = Status::new(protocol, "A Minecraft Server".to_string());
        let mut payload = Vec::new();

        write_string(
            &mut payload,
            &serde_json::to_string(&server_description).unwrap(),
        );

        // write length and id, then send
        write_varint(w, payload.len() as i32 + 1);
        write_varint(w, 0x00);

        w.write_all(&payload).unwrap();
    }
}

/// Ping packet received after StatusResponse
pub struct Ping {
    pub _header: PacketHeader,
    pub payload: i64,
}

impl Ping {
    pub fn new<R: Read>(r: &mut R) -> Self {
        let _header = read_header(r);
        let payload = read(r);
        Self { _header, payload }
    }
}

pub struct Pong;

impl Pong {
    pub fn send<W: Write>(w: &mut W, payload: i64) {
        let pong = &payload.to_be_bytes();

        write_varint(w, pong.len() as i32 + 1);
        write_varint(w, 0x01);
        w.write_all(pong).unwrap();
    }
}

/// Login Start packet received from client
pub struct LoginStart {
    pub _header: PacketHeader,
    pub player_name: String,
    pub uuid: Uuid,
}

impl LoginStart {
    pub fn new<R: Read>(r: &mut R) -> Self {
        let _header = read_header(r);
        let player_name = read_string(r);
        let uuid = Uuid::from_u128_le(read(r));
        Self {
            _header,
            player_name,
            uuid,
        }
    }
}

/// Ping packet sent by legacy clients
pub struct LegacyPing {
    pub hostname: String,
    pub version: u8,
    pub mc_version: String,
    pub port: i32,
}

impl LegacyPing {
    pub fn new<R: Read>(r: &mut R) -> Self {
        let _id: u8 = read(r);
        let _ping_payload: u8 = read(r);
        let _plugin_id: u8 = read(r);

        let _mc_pinghost = read_legacy_string(r);
        let _remaining_length: i16 = read(r);
        let version = read(r);
        let hostname = read_legacy_string(r);
        let port = read(r);
        Self {
            hostname,
            version,
            mc_version: mc_version_legacy(version).to_string(),
            port,
        }
    }
}

pub struct LegacyPingResponse;

impl LegacyPingResponse {
    pub fn send<W: Write>(w: &mut W) {
        let payload = [
            0xFF, // packet id
            0x00, 0x23, // length of packet
            0x00, 0xA7, 0x00, 0x31, 0x00, 0x00, // magic prefix
            0x00, 0x37, 0x00, 0x38, 0x00, 0x00, // protocol version 78
            0x00, 0x31, 0x00, 0x2E, 0x00, 0x36, 0x00, 0x2E, 0x00, 0x34, 0x00,
            0x00, // version 1.6.4
            0x00, 0x41, 0x00, 0x20, 0x00, 0x4D, 0x00, 0x69, 0x00, 0x6E, 0x00, 0x65, 0x00, 0x63,
            0x00, 0x72, 0x00, 0x61, 0x00, 0x66, 0x00, 0x74, 0x00, 0x20, 0x00, 0x53, 0x00, 0x65,
            0x00, 0x72, 0x00, 0x76, 0x00, 0x65, 0x00, 0x72, 0x00,
            0x00, // "A Minecraft Server"
            0x00, 0x30, 0x00, 0x00, // 0 current players
            0x00, 0x32, 0x00, 0x30, // 20 max players
        ];
        w.write_all(&payload).unwrap();
    }
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
            players: StatusPlayers { max: 20, online: 0 },
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
}

#[derive(Serialize)]
struct StatusDescription {
    pub text: String,
}
