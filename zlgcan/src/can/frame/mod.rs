mod common;
pub use common::{ZCanFrameType, ZCanTxMode};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub(crate) use linux::*;
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub(crate) use windows::*;

#[repr(C)]
#[derive(Copy, Clone)]
pub union ZCanFrame {
    pub(crate) can: ZCanFrameInner,
    pub(crate) canfd: ZCanFdFrameInner,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union ZCanChlError {
    pub(crate) v1: common::ZCanChlErrorInner,
    #[cfg(target_os = "linux")]
    pub(crate) v2: ZCanChlErrInfo,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct USBCanEUAutoTransFrame {
    pub interval: u32,
    pub can_id: u32,
    pub is_extend: bool,
    pub is_remote: bool,
    pub length: u8,
    pub data: *const u8,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct USBCanEUWhiteList {
    pub is_extend: bool,
    pub start: u32,
    pub stop: u32,
}

