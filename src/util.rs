use std::io::{Read, Write};

use bytemuck::{AnyBitPattern, NoUninit};
use log::warn;

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

pub fn write_varint<W: Write>(w: &mut W, value: i32) {
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
    let len = read_varint(r) as usize;
    let mut buf = vec![0u8; len];
    r.read_exact(&mut buf).unwrap();
    String::from_utf8(buf).unwrap()
}

pub fn read_legacy_string<R: Read>(r: &mut R) -> String {
    let len: i16 = read(r);
    let mut buf = vec![0u8; len as usize];
    r.read_exact(&mut buf).unwrap();
    String::from_utf8(buf).unwrap()
}

pub fn write_string<W: Write>(w: &mut W, s: &str) {
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

pub fn write<W: Write, T: NoUninit>(w: &mut W, value: &T) {
    let mut buffer = bytemuck::bytes_of(value).to_vec();

    // network bytes are in big endian
    #[cfg(target_endian = "little")]
    buffer.reverse();

    w.write_all(&buffer).unwrap();
}
