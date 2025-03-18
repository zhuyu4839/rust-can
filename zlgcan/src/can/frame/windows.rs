use std::ffi::{c_uchar, c_uint, c_ulonglong};
use rs_can::{CanDirect, IdentifierFlags, EFF_MASK, MAX_FD_FRAME_SIZE, MAX_FRAME_SIZE};
use crate::can::{CanMessage, constant::{CANFD_BRS, CANFD_ESI}};
use super::common::ZCanMsg20;

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub(crate) struct ZCanFrameTx {
    pub(crate) frame: ZCanMsg20<MAX_FRAME_SIZE>,
    pub(crate) tx_mode: c_uint, // ZCanTxMode
}

impl From<CanMessage> for ZCanFrameTx {
    fn from(msg: CanMessage) -> Self {
        let can_id = can_id_add_flags(&msg);
        let tx_mode = msg.tx_mode() as u32;
        let length = msg.data.len() as u8;
        let mut data = msg.data;
        data.resize(MAX_FRAME_SIZE, Default::default());
        Self {
            tx_mode,
            frame: ZCanMsg20::new(can_id, length, Default::default(), data.try_into().unwrap())
        }
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
        let can_id = self.frame.can_id;
        let length = self.frame.can_len as usize;
        let mut data = self.frame.data.to_vec();
        data.resize(length, Default::default());
        CanMessage {
            timestamp: self.timestamp,
            arbitration_id: can_id & EFF_MASK,
            is_extended_id: (can_id & IdentifierFlags::EXTENDED.bits()) > 0,
            is_remote_frame: (can_id & IdentifierFlags::REMOTE.bits()) > 0,
            is_error_frame: (can_id & IdentifierFlags::ERROR.bits()) > 0,
            channel: self.frame.__res0,
            length,
            data,
            is_fd: false,
            direct: CanDirect::Receive,
            bitrate_switch: false,
            error_state_indicator: false,
            tx_mode: None,
        }
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
        let can_id = can_id_add_flags(&msg);
        let tx_mode = msg.tx_mode() as u32;
        let length = msg.data.len() as u8;
        let flags = c_uchar::default() |
            if msg.bitrate_switch { CANFD_BRS } else { Default::default() } |
            if msg.error_state_indicator { CANFD_ESI } else { Default::default() };
        let mut data = msg.data;
        data.resize(MAX_FRAME_SIZE, Default::default());
        Self {
            tx_mode,
            frame: ZCanMsg20::new(can_id, length, flags, data.try_into().unwrap())
        }
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
        let can_id = self.frame.can_id;
        let length = self.frame.can_len as usize;
        let mut data = Vec::from(&self.frame.data);
        data.resize(MAX_FD_FRAME_SIZE, Default::default());
        CanMessage {
            timestamp: self.timestamp,
            arbitration_id: can_id & EFF_MASK,
            is_extended_id: (can_id & IdentifierFlags::EXTENDED.bits()) > 0,
            is_remote_frame: (can_id & IdentifierFlags::REMOTE.bits()) > 0,
            is_error_frame: (can_id & IdentifierFlags::ERROR.bits()) > 0,
            channel: self.frame.__res0,
            length,
            data,
            is_fd: true,
            direct: CanDirect::Receive,
            bitrate_switch: self.frame.flags & CANFD_BRS > 0,
            error_state_indicator: self.frame.flags & CANFD_ESI > 0,
            tx_mode: None,
        }
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

// pub(crate) type ZCanChlError = ZCanChlErrorInner;
fn can_id_add_flags(msg: &CanMessage) -> u32 {
    msg.arbitration_id |
        if msg.is_extended_id { IdentifierFlags::EXTENDED.bits() } else { Default::default() } |
        if msg.is_remote_frame { IdentifierFlags::REMOTE.bits() } else { Default::default() } |
        if msg.is_error_frame { IdentifierFlags::ERROR.bits() } else { Default::default() }
}
