use std::ffi::{c_uint, c_ulonglong};
use rs_can::{MAX_FD_FRAME_SIZE, MAX_FRAME_SIZE};
use crate::can::CanMessage;
use super::common::ZCanMsg20;

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub(crate) struct ZCanFrameTx {
    pub(crate) frame: ZCanMsg20<MAX_FRAME_SIZE>,
    pub(crate) tx_mode: c_uint, // ZCanTxMode
}

impl From<CanMessage> for ZCanFrameTx {
    fn from(msg: CanMessage) -> Self {
        let tx_mode = msg.tx_mode() as u32;
        let frame = msg.into();
        Self { frame, tx_mode, }
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub(crate) struct ZCanFrameRx {
    pub(crate) frame: ZCanMsg20<MAX_FRAME_SIZE>,
    pub(crate) timestamp: c_ulonglong,
}

impl Into<CanMessage> for ZCanFrameRx {
    fn into(self) -> CanMessage {
        let timestamp = self.timestamp;
        let mut msg: CanMessage = self.frame.into();
        msg.timestamp = timestamp;

        msg
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub(crate) struct ZCanFdFrameTx {
    pub(crate) frame: ZCanMsg20<MAX_FD_FRAME_SIZE>,
    pub(crate) tx_mode: c_uint, // ZCanTxMode
}

impl From<CanMessage> for ZCanFdFrameTx {
    fn from(msg: CanMessage) -> Self {
        let tx_mode = msg.tx_mode() as u32;
        let frame = msg.into();
        Self { frame, tx_mode, }
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub(crate) struct ZCanFdFrameRx {
    pub(crate) frame: ZCanMsg20<MAX_FD_FRAME_SIZE>,
    pub(crate) timestamp: c_ulonglong,
}

impl Into<CanMessage> for ZCanFdFrameRx {
    fn into(self) -> CanMessage {
        let timestamp = self.timestamp;
        let mut msg: CanMessage = self.frame.into();
        msg.timestamp = timestamp;

        msg
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) union ZCanFrameInner {
    pub(crate) tx: ZCanFrameTx,
    pub(crate) rx: ZCanFrameRx,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) union ZCanFdFrameInner {
    pub(crate) tx: ZCanFdFrameTx,
    pub(crate) rx: ZCanFdFrameRx,
}
