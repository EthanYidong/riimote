use bitflags::bitflags;

use std::convert::TryInto;

bitflags! {
    pub struct ButtonFlags: u16 {
        const D_PAD_LEFT = fb(0);
        const D_PAD_RIGHT = fb(1);
        const D_PAD_DOWN = fb(2);
        const D_PAD_UP = fb(3);
        const PLUS = fb(4);
        const TWO = sb(0);
        const ONE = sb(1);
        const B = sb(2);
        const A = sb(3);
        const MINUS = sb(4);
        const HOME = sb(7);
    }
}

impl ButtonFlags {
    pub fn from_bytes(bytes: &[u8]) -> Result<ButtonFlags, std::array::TryFromSliceError> {
        Ok(ButtonFlags::from_bits_truncate(u16::from_be_bytes(
            bytes.try_into()?,
        )))
    }
}

const fn fb(first: u8) -> u16 {
    u16::from_be_bytes([1 << first, 0x00])
}

const fn sb(second: u8) -> u16 {
    u16::from_be_bytes([0x00, 1 << second])
}
