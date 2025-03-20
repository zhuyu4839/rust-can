use std::time::{SystemTime, UNIX_EPOCH};
use crate::CanType;
use crate::constants::{DEFAULT_PADDING, MAX_FRAME_SIZE, MAX_FD_FRAME_SIZE, MAX_XL_FRAME_SIZE};
use crate::error::Error;
use crate::frame::Type;

/// resize data with default padding.
#[inline]
pub fn data_resize(data: &mut Vec<u8>, size: usize) {
    data.resize(size, DEFAULT_PADDING);
}

#[inline]
pub fn can_type(len: usize) -> Result<Type, Error> {
    match len {
        ..=MAX_FRAME_SIZE => Ok(Type::Can),
        ..=MAX_FD_FRAME_SIZE => Ok(Type::CanFd),
        ..=MAX_XL_FRAME_SIZE => Ok(Type::CanXl),
        _ => Err(Error::OtherError("length of frame is out of range!".into())),
    }
}

/// get CAN dlc
#[inline]
pub fn can_dlc(length: usize, r#type: Type) -> isize {
    match r#type {
        CanType::Can => match length {
            ..=MAX_FRAME_SIZE => length as isize,
            _ => -1,
        },
        CanType::CanFd =>  match length {
            ..=MAX_FRAME_SIZE => length as isize,
            9..=12 =>  12,
            13..=16 => 16,
            17..=20 => 20,
            21..=24 => 24,
            25..=32 => 32,
            33..=48 => 48,
            49..=MAX_FD_FRAME_SIZE => 64,
            _ => -1,
        },
        Type::CanXl => -1,
    }
}

#[inline]
pub fn system_timestamp() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(v) => v.as_millis() as u64,
        Err(e) => {
            log::warn!("RUST-CAN - SystemTimeError: {0} when conversion failed!", e);
            0
        }
    }
}
