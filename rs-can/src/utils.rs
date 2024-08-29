use crate::constant::DEFAULT_PADDING;
use std::time::{SystemTime, UNIX_EPOCH};

/// Get system timestamp(ms)
#[inline]
pub fn system_timestamp() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(v) => v.as_millis() as u64,
        Err(e) => {
            log::warn!("RS-CAN - SystemTimeError: {0} when conversion failed!", e);
            0
        }
    }
}

/// resize data with default padding.
#[inline]
pub fn data_resize(data: &mut Vec<u8>, size: usize) {
    data.resize(size, DEFAULT_PADDING);
}
