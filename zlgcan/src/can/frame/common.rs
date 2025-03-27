use std::ffi::{c_uchar, c_uint};
use std::fmt::{Display, Formatter};
use rs_can::{can_utils, CanDirect, CanError, CanType, IdentifierFlags, DEFAULT_PADDING, EFF_MASK, MAX_FRAME_SIZE};
use crate::can::{CanMessage, constant::{CANFD_BRS, CANFD_ESI}};

/// Then CAN frame type used in crate.
#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
pub enum ZCanFrameType {
    CAN = 0,
    CANFD = 1,
    ALL = 2,
}

impl TryFrom<u8> for ZCanFrameType {
    type Error = CanError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ZCanFrameType::CAN),
            1 => Ok(ZCanFrameType::CANFD),
            2 => Ok(ZCanFrameType::ALL),
            _ => Err(CanError::other_error("parameter not supported")),
        }
    }
}

impl Display for ZCanFrameType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CAN => write!(f, "CAN"),
            Self::CANFD => write!(f, "CANFD"),
            Self::ALL => write!(f, "CAN|CANFD"),
        }
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub enum ZCanTxMode {
    #[default]
    Normal = 0,             //**< normal transmission */
    Once = 1,               //**< single-shot transmission */
    SelfReception = 2,      //**< self reception */
    SelfReceptionOnce = 3,  //**< single-shot transmission & self reception */
}

impl TryFrom<u8> for ZCanTxMode {
    type Error = CanError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ZCanTxMode::Normal),
            1 => Ok(ZCanTxMode::Once),
            2 => Ok(ZCanTxMode::SelfReception),
            3 => Ok(ZCanTxMode::SelfReceptionOnce),
            _ => Err(CanError::other_error("parameter not supported")),
        }
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub(crate) struct ZCanChlErrorInner {
    pub(crate) code: c_uint,
    pub(crate) passive: [c_uchar; 3],
    pub(crate) arb_lost: c_uchar,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct ZCanMsg20<const S: usize> {
    pub(crate) can_id: c_uint,
    pub(crate) can_len: c_uchar,
    pub(crate) flags: c_uchar,  /* padding when using can else additional flags for CAN FD,i.e error code */
    pub(crate) __res0: c_uchar, /* reserved / padding (used for channel) */
    #[allow(dead_code)]
    pub(crate) __res1: c_uchar, /* reserved / padding */
    pub(crate) data: [c_uchar; S],
}

impl<const S: usize> ZCanMsg20<S> {
    pub fn new(
        can_id: c_uint,
        can_len: c_uchar,
        flags: c_uchar,
        data: [c_uchar; S],
    ) -> Self {
        Self {
            can_id,
            can_len,
            flags,
            __res0: Default::default(),
            __res1: Default::default(),
            data
        }
    }

    #[inline(always)]
    pub fn set_channel(&mut self, channel: u8) -> &Self {
        self.__res0 = channel;
        self
    }
    #[allow(unused)]
    #[inline(always)]
    pub fn get_channel(&self) -> u8 {
        self.__res0
    }
}

impl<const S: usize> Default for ZCanMsg20<S> {
    fn default() -> Self {
        Self {
            can_id: Default::default(),
            can_len: Default::default(),
            flags: Default::default(),
            __res0: Default::default(),
            __res1: Default::default(),
            data: [Default::default(); S],
        }
    }
}

impl<const S: usize> Into<CanMessage> for ZCanMsg20<S> {
    fn into(self) -> CanMessage {
        let can_type = can_utils::can_type(S).unwrap();

        let can_id = self.can_id;
        let length = self.can_len as usize;
        let mut data = self.data.to_vec();
        data.resize(length, Default::default());
        CanMessage {
            timestamp: Default::default(),
            arbitration_id: can_id & EFF_MASK,
            is_extended_id: (can_id & IdentifierFlags::EXTENDED.bits()) > 0,
            is_remote_frame: (can_id & IdentifierFlags::REMOTE.bits()) > 0,
            is_error_frame: (can_id & IdentifierFlags::ERROR.bits()) > 0,
            channel: self.__res0,
            length,
            data,
            can_type,
            direct: CanDirect::Receive,
            bitrate_switch: match can_type {
                CanType::Can => false,
                CanType::CanFd => self.flags & CANFD_BRS > 0,
                CanType::CanXl => todo!(),
            },
            error_state_indicator: match can_type {
                CanType::Can => false,
                CanType::CanFd => self.flags & CANFD_ESI > 0,
                CanType::CanXl => todo!(),
            },
            tx_mode: None,
        }
    }
}

impl<const S: usize> From<CanMessage> for ZCanMsg20<S> {
    fn from(msg: CanMessage) -> Self {
        let is_fd = S > MAX_FRAME_SIZE;

        let can_id = can_id_add_flags(&msg);
        let length = msg.data.len() as u8;
        let flags = if is_fd {
            (if msg.bitrate_switch { CANFD_BRS } else { Default::default() }) |
            (if msg.error_state_indicator { CANFD_ESI } else { Default::default() })
        }
        else {
            Default::default()
        };
        let mut data = msg.data;
        data.resize(S, DEFAULT_PADDING);

        Self::new(can_id, length, flags, data.try_into().unwrap())
    }
}

// pub(crate) type ZCanChlError = ZCanChlErrorInner;
fn can_id_add_flags(msg: &CanMessage) -> u32 {
    msg.arbitration_id |
        if msg.is_extended_id { IdentifierFlags::EXTENDED.bits() } else { Default::default() } |
        if msg.is_remote_frame { IdentifierFlags::REMOTE.bits() } else { Default::default() } |
        if msg.is_error_frame { IdentifierFlags::ERROR.bits() } else { Default::default() }
}
