use std::ffi::{c_uint, c_ulonglong};
use rs_can::{MAX_FD_FRAME_SIZE, MAX_FRAME_SIZE};
use crate::can::CanMessage;
use super::common::ZCanMsg20;

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub(crate) struct ZCanFrameTx<const S: usize> {
    pub(crate) frame: ZCanMsg20<S>,
    pub(crate) tx_mode: c_uint, // ZCanTxMode
}

impl<const S: usize> From<CanMessage> for ZCanFrameTx<S> {
    fn from(msg: CanMessage) -> Self {
        let tx_mode = msg.tx_mode() as u32;
        let frame = msg.into();
        Self { frame, tx_mode, }
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub(crate) struct ZCanFrameRx<const S: usize> {
    pub(crate) frame: ZCanMsg20<S>,
    pub(crate) timestamp: c_ulonglong,
}

impl<const S: usize> Into<CanMessage> for ZCanFrameRx<S> {
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
    pub(crate) tx: ZCanFrameTx<MAX_FRAME_SIZE>,
    pub(crate) rx: ZCanFrameRx<MAX_FRAME_SIZE>,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) union ZCanFdFrameInner {
    pub(crate) tx: ZCanFrameTx<MAX_FD_FRAME_SIZE>,
    pub(crate) rx: ZCanFrameRx<MAX_FD_FRAME_SIZE>,
}
