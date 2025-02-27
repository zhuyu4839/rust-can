use std::ffi::{c_char, CStr};
use rs_can::CanError;
use rs_can::utils::system_timestamp;

#[inline]
pub fn c_str_to_string(src: *const c_char) -> Result<String, CanError> {
    if src.is_null() {
        Err(CanError::OtherError("null pointer".to_string()))
    } else {
        let c_str = unsafe { CStr::from_ptr(src) };
        let s_slice = c_str.to_str().map_err(|e| CanError::OtherError(e.to_string()))?;
        let value = String::from(s_slice);

        Ok(value)
    }
}

#[inline]
pub fn fix_system_time(frame_timestamp: u64, fix_timestamp: u64) -> u64 {
    frame_timestamp + fix_timestamp
}

#[inline]
pub fn fix_device_time(fix_timestamp: u64) -> u64 {
    system_timestamp() - fix_timestamp
}
