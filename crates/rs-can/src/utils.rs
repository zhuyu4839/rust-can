use std::time::{SystemTime, UNIX_EPOCH};
use iso15765_2::can::DEFAULT_PADDING;

/// Get system timestamp(ms)
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

/// resize data with default padding.
#[inline]
pub fn data_resize(data: &mut Vec<u8>, size: usize) {
    data.resize(size, DEFAULT_PADDING);
}
