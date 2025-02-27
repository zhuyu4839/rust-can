use std::time::{SystemTime, UNIX_EPOCH};
use crate::{CanError, CANFD_FRAME_MAX_SIZE, CAN_FRAME_MAX_SIZE, DEFAULT_PADDING};

/// Get system timestamp(ms)
#[inline]
pub fn system_timestamp() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(v) => v.as_millis() as u64,
        Err(e) => {
            log::warn!("RUST-CAN - SystemTimeError: {} when conversion failed!", e);
            0
        }
    }
}

/// resize data with default padding.
#[inline]
pub fn data_resize(data: &mut Vec<u8>, size: usize) {
    data.resize(size, DEFAULT_PADDING);
}

#[inline]
pub fn is_can_fd_len(len: usize) -> Result<bool, CanError> {
    match len {
        ..=CAN_FRAME_MAX_SIZE => Ok(false),
        ..=CANFD_FRAME_MAX_SIZE => Ok(true),
        _ => Err(CanError::DataOutOfRange(len)),
    }
}

/// get CAN dlc
#[inline]
pub fn can_dlc(length: usize, fd: bool) -> Option<usize> {
    if fd {
        match length {
            ..=CAN_FRAME_MAX_SIZE => Some(length),
            9..=12 =>  Some(12),
            13..=16 => Some(16),
            17..=20 => Some(20),
            21..=24 => Some(24),
            25..=32 => Some(32),
            33..=48 => Some(48),
            49..=CANFD_FRAME_MAX_SIZE => Some(64),
            _ => None,
        }
    }
    else {
        match length {
            ..=CAN_FRAME_MAX_SIZE => Some(length),
            _ => None,
        }
    }
}
