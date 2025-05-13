use std::time::{SystemTime, UNIX_EPOCH};
use std::fmt;

pub struct UUID([u8; 16]);

impl fmt::Display for UUID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}-{:x}-{:x}-{:x}-{:x}",
            u32::from_be_bytes([self.0[0], self.0[1], self.0[2], self.0[3]]),
            u16::from_be_bytes([self.0[4], self.0[5]]),
            u16::from_be_bytes([self.0[6] & 0x0f | 0x40, self.0[7]]),
            u16::from_be_bytes([self.0[8] & 0x3f | 0x80, self.0[9]]),
            u64::from_be_bytes([self.0[10], self.0[11], self.0[12], self.0[13], self.0[14], self.0[15], 0, 0])
        )
    }
}

pub fn generate_uuid() -> UUID {
    let mut bytes = [0u8; 16];
    let start = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u128;
    for i in 0..16 {
        bytes[i] = ((start >> (i * 8)) & 0xFF) as u8;
    }
    UUID(bytes)
}
