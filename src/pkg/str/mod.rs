use crc::{Crc, CRC_32_ISO_HDLC};

pub fn generate_short_hash(input: &str) -> String {
    let crc = Crc::<u32>::new(&CRC_32_ISO_HDLC);
    let hash = crc.checksum(input.as_bytes());

    let hex_hash = format!("{:x}", hash);
    hex_hash[..6].to_string()
}