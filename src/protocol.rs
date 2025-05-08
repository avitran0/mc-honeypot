use std::io::{Read, Write};

use bytemuck::AnyBitPattern;
use log::warn;

pub struct PacketHeader {
    pub length: i32,
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
    let mut num_read = 0;
    let mut result = String::with_capacity(16);
    loop {
        let mut byte = [0u8];
        if r.read_exact(&mut byte).is_err() {
            break;
        }
        let val = byte[0];
        if val == 0 {
            break;
        }
        result.push(val as char);
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
    bytemuck::try_from_bytes(&buffer)
        .copied()
        .unwrap_or_default()
}

pub fn read_header<R: Read>(r: &mut R) -> PacketHeader {
    PacketHeader {
        length: read_varint(r),
        id: read_varint(r),
    }
}

pub fn send_status<S: Read + Write>(s: &mut S) {
    let header = read_header(s);
    if header.id != 0x00 {
        warn!("packet id mismatch");
        return;
    }

    let server_description = r#"{
        "version": { "name": "Rust-Server", "protocol": 754 },
        "players": { "max": 100, "online": 0, "sample": [] },
        "description": { "text": "Hello from Rust!" }
    }"#;
    let mut payload = Vec::new();
    write_varint(&mut payload, 0x00);
    write_string(&mut payload, server_description);

    // write length and send
    write_varint(s, payload.len() as i32);
    s.write_all(&payload).unwrap();

    // ping packet
    let header = read_header(s);
    let mut ping_payload = [0u8; 8];
    s.read_exact(&mut ping_payload).unwrap();

    // echo back ping payload as pong
    let mut pong = Vec::new();
    write_varint(&mut pong, 0x01);
    pong.extend_from_slice(&ping_payload);

    write_varint(s, pong.len() as i32);
    s.write_all(&pong).unwrap();
}

pub fn send_login<R: Read>(r: &mut R) {}

pub const fn mc_version(protocol_version: i32) -> &'static str {
    match protocol_version {
        770 => "1.21.5",
        769 => "1.21.4",
        768 => "1.21.3",
        767 => "1.21.1",
        766 => "1.20.6",
        765 => "1.20.4",
        764 => "1.20.2",
        _ => "unknown",
    }
}
