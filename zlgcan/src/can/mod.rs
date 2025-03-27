mod channel;
pub(crate) mod constant;
mod frame;
mod message;
// mod util;

pub use channel::*;
pub use frame::*;
pub use message::*;

use rs_can::CanError;

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
pub enum ZCanFdStd {
    CANFD_ISO = 0,
    CANFD_NON_ISO = 1,
}

impl TryFrom<u8> for ZCanFdStd {
    type Error = CanError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ZCanFdStd::CANFD_ISO),
            1 => Ok(ZCanFdStd::CANFD_NON_ISO),
            _ => Err(CanError::other_error("parameter not supported")),
        }
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub enum ZCanFilterType {
    #[default]
    Double = 0,
    Single = 1,
}

impl TryFrom<u8> for ZCanFilterType {
    type Error = CanError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ZCanFilterType::Double),
            1 => Ok(ZCanFilterType::Single),
            _ => Err(CanError::other_error("parameter not supported")),
        }
    }
}

/// The reference for Linux device
pub enum Reference {
    Filter = 0x14,          // filter setting; @see ZCAN_Filter and ZCanFilterTable
    SkdSend = 0x16,         // timed send setting; @see ZCAN_TTX
    SkdSendStatus = 0x17,   // timed send status; 0-disable, 1-enable
    Resistance = 0x18,      // terminal resistance; 0-disable, 1-enable
    Timeout = 0x44,         // send timeout; range 0~4000ms
}

