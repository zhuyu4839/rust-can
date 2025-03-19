use std::time::{SystemTime, UNIX_EPOCH};

use crate::constants::{DEFAULT_PADDING, MAX_FD_FRAME_SIZE, MAX_FRAME_SIZE};
use crate::error::Error;

/// resize data with default padding.
#[inline]
pub fn data_resize(data: &mut Vec<u8>, size: usize) {
    data.resize(size, DEFAULT_PADDING);
}

#[inline]
pub fn is_can_fd_len(len: usize) -> Result<bool, Error> {
    match len {
        ..=MAX_FRAME_SIZE => Ok(false),
        ..=MAX_FD_FRAME_SIZE => Ok(true),
        _ => Err(Error::OtherError("length of frame is out of range!".into())),
    }
}

/// get CAN dlc
#[inline]
pub fn can_dlc(length: usize, fd: bool) -> Option<usize> {
    if fd {
        match length {
            ..=MAX_FRAME_SIZE => Some(length),
            9..=12 =>  Some(12),
            13..=16 => Some(16),
            17..=20 => Some(20),
            21..=24 => Some(24),
            25..=32 => Some(32),
            33..=48 => Some(48),
            49..=MAX_FD_FRAME_SIZE => Some(64),
            _ => None,
        }
    }
    else {
        match length {
            ..=MAX_FRAME_SIZE => Some(length),
            _ => None,
        }
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
