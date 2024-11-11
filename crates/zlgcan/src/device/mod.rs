mod dev;
mod property;
mod typedef;

pub use dev::*;
pub use property::*;
pub use typedef::*;

/// The information about derive device.
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct DeriveInfo {
    pub(crate) canfd: bool,
    pub(crate) channels: u8,
    // pub(crate) resistance: bool,
}

impl DeriveInfo {
    pub fn new(canfd: bool, channels: u8) -> Self {
        Self { canfd, channels }
    }
}


