use std::ffi::{c_uchar, c_uint, c_ushort};
use rs_can::{CanDirect, DEFAULT_PADDING, MAX_FD_FRAME_SIZE, MAX_FRAME_SIZE};
use crate::can::{CanMessage, constant::{TIME_FLAG_VALID, CANERR_FRAME_LENGTH}};

/// only used usbcan on linux
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub(crate) struct ZCanFrameVCI {
    pub(crate) can_id: c_uint,
    pub(crate) timestamp: c_uint,
    pub(crate) time_flag: c_uchar,
    pub(crate) tx_mode: c_uchar,
    pub(crate) rem_flag: c_uchar,
    pub(crate) ext_flag: c_uchar,
    pub(crate) can_len: c_uchar,
    pub(crate) data: [c_uchar; MAX_FRAME_SIZE],
    pub(crate) channel: c_uchar,
    #[allow(dead_code)]
    pub(crate) reserved: [c_uchar; 3-1],    // use 1byte to channel
}

impl Into<CanMessage> for ZCanFrameVCI {
    fn into(self) -> CanMessage {
        if self.time_flag != TIME_FLAG_VALID {
            log::warn!("ZCanFrameVCI - time flag is invalid")
        }
        let timestamp = self.timestamp as u64;
        let arbitration_id = self.can_id;
        let is_extended_id = self.ext_flag > 0;
        let is_remote_frame = self.rem_flag > 0;
        let channel = self.channel;
        let length = self.can_len as usize;
        let mut data = self.data.to_vec();
        data.resize(length, Default::default());
        CanMessage {
            timestamp,
            arbitration_id,
            is_extended_id,
            is_remote_frame,
            is_error_frame: false,
            channel,
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

impl From<CanMessage> for ZCanFrameVCI {
    fn from(msg: CanMessage) -> Self {
        let can_id = msg.arbitration_id;
        let timestamp = msg.timestamp as u32;
        let time_flag = TIME_FLAG_VALID;
        let tx_mode = msg.tx_mode();
        let rem_flag = if msg.is_remote_frame { 1 } else { 0 };
        let ext_flag = if msg.is_extended_id { 1 } else { 0 };
        let can_len = msg.length as u8;
        let channel = msg.channel;
        let mut data = msg.data;
        data.resize(MAX_FRAME_SIZE, DEFAULT_PADDING);
        Self {
            can_id,
            timestamp,
            time_flag,
            tx_mode,
            rem_flag,
            ext_flag,
            can_len,
            data: data.try_into().unwrap(),
            channel,
            reserved: Default::default(),
        }
    }
}

/// only used usbcanfd on linux
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct ZCanMsg20<const S: usize> {
    pub(crate) timestamp: c_uint,
    pub(crate) can_id: c_uint,
    /// bit31~28: /**< TX-mode, @see ZCAN_TX_MODE */
    /// bit27~24: /**< 0-CAN2.0, 1-CANFD */
    /// bit23   : /**< 0-data_frame, 1-remote_frame */
    /// bit22   : /**< 0-std_frame, 1-ext_frame */
    /// bit21   : /**< error flag */
    /// bit20   : /**< bit-rate switch */
    /// bit19   : /**< error state */
    /// bit18~00: reserved
    pub(crate) flags: c_uint,
    pub(crate) __pad: c_ushort,
    pub(crate) channel: c_uchar,
    pub(crate) can_len: c_uchar,
    pub(crate) data: [c_uchar; S],
}

impl<const S: usize> Default for ZCanMsg20<S> {
    fn default() -> Self {
        Self {
            timestamp: Default::default(),
            can_id: Default::default(),
            flags: Default::default(),
            __pad: Default::default(),
            channel: Default::default(),
            can_len: Default::default(),
            data: [Default::default(); S],
        }
    }
}

impl<const S: usize> Into<CanMessage> for ZCanMsg20<S> {
    fn into(self) -> CanMessage {
        let length = self.can_len as usize;
        let mut data = self.data.to_vec();
        data.resize(length, Default::default());
        let is_fd = (self.flags & (0x03 << 24)) >> 24;
        let is_fd = match is_fd {
            0x00 => false,
            0x01 => true,
            v => {
                log::warn!("ZCanMsg20 - unexpected value: {} to set can-fd", v);
                false
            }
        };
        CanMessage {
            timestamp: self.timestamp as u64,
            arbitration_id: self.can_id,
            is_extended_id: (self.flags & (0x01 << 22)) > 0,
            is_remote_frame: (self.flags & (0x01 << 23)) > 0,
            is_error_frame: (self.flags & (0x01 << 21)) > 0,
            channel: self.channel,
            length,
            data,
            is_fd,
            direct: CanDirect::Receive,
            bitrate_switch: (self.flags & (0x01 << 20)) > 0,
            error_state_indicator: (self.flags & (0x01 << 19)) > 0,
            tx_mode: Some(((self.flags & (0x3 << 28)) >> 28) as u8),
        }
    }
}

impl<const S: usize> From<CanMessage> for ZCanMsg20<S> {
    fn from(msg: CanMessage) -> Self {
        let flags = (msg.tx_mode() as u32 >> 28) |
            if msg.is_fd { 0x01u32 >> 24 } else { 0 } |
            if msg.is_remote_frame { 0x01u32 >> 23 } else { 0 } |
            if msg.is_extended_id { 0x01u32 >> 22 } else { 0 } |
            if msg.is_error_frame { 0x01u32 >> 21 } else { 0 } |
            if msg.bitrate_switch { 0x01u32 >> 20 } else { 0 } |
            if msg.error_state_indicator { 0x01u32 >> 19 } else { 0 };
        let timestamp = msg.timestamp as u32;
        let can_id = msg.arbitration_id;
        let channel = msg.channel;
        let can_len = msg.length as u8;
        let mut data = msg.data;
        data.resize(S, DEFAULT_PADDING);
        Self {
            timestamp,
            can_id,
            flags,
            __pad: Default::default(),
            channel,
            can_len,
            data: data.try_into().unwrap(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) union ZCanFrameInner {
    pub(crate) usbcan: ZCanFrameVCI,   // libusbcan.so
    pub(crate) usbcanfd: ZCanMsg20<MAX_FRAME_SIZE>, // libusbcanfd.so
    pub(crate) other: super::common::ZCanMsg20<MAX_FRAME_SIZE>,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) union ZCanFdFrameInner {
    pub(crate) usbcanfd: ZCanMsg20<MAX_FD_FRAME_SIZE>,  // libusbcanfd.so
    pub(crate) other: super::common::ZCanMsg20<MAX_FD_FRAME_SIZE>,
}

/// only used usbcanfd on linux
pub(crate) type ZCanChlErrInfo = ZCanMsg20<CANERR_FRAME_LENGTH>;
