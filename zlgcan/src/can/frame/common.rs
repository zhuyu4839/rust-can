use std::ffi::{c_uchar, c_uint};
use std::fmt::{Display, Formatter};
use rs_can::CanError;

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
            _ => Err(CanError::OtherError("parameter not supported".to_owned())),
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
            _ => Err(CanError::OtherError("parameter not supported".to_owned())),
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
    // pub fn new(
    //     can_id: c_uint,
    //     can_len: c_uchar,
    //     data: [c_uchar; S],
    // ) -> Self {
    //     Self::new_fd(can_id, can_len, Default::default(), data)
    // }
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

// #[derive(Debug, Copy, Clone)]
// pub(crate) enum ZCanHdrInfoField {
//     TxMode = 1,
//     FrameType = 2,
//     IsRemoteFrame = 3,
//     IsExtendFrame = 4,
//     IsErrorFrame = 5,
//     IsBitrateSwitch = 6,
//     IsErrorStateIndicator = 7,
// }
//
// impl TryFrom<u8> for ZCanHdrInfoField {
//     type Error = CanError;
//     fn try_from(value: u8) -> Result<Self, Self::Error> {
//         match value {
//             1 => Ok(ZCanHdrInfoField::TxMode),
//             2 => Ok(ZCanHdrInfoField::FrameType),
//             3 => Ok(ZCanHdrInfoField::IsRemoteFrame),
//             4 => Ok(ZCanHdrInfoField::IsExtendFrame),
//             5 => Ok(ZCanHdrInfoField::IsErrorFrame),
//             6 => Ok(ZCanHdrInfoField::IsBitrateSwitch),
//             7 => Ok(ZCanHdrInfoField::IsErrorStateIndicator),
//             _ => Err(CanError::OtherError("parameter not supported".to_owned())),
//         }
//     }
// }
//
// #[repr(C)]
// #[derive(Debug, Default, Copy, Clone)]
// pub(crate) struct ZCanHdrInfo {
//     mode: c_uchar,  // U8 txm : 4; /**< TX-mode, @see ZCAN_TX_MODE */
//     // U8 fmt : 4; /**< 0-CAN2.0, 1-CANFD */
//     flag: c_uchar,  // U8 sdf : 1; /**< 0-data_frame, 1-remote_frame */
//                     // U8 sef : 1; /**< 0-std_frame, 1-ext_frame */
//                     // U8 err : 1; /**< error flag */
//                     // U8 brs : 1; /**< bit-rate switch */
//                     // U8 est : 1; /**< error state */
//                     // 5~7bit not used
//     #[allow(dead_code)]
//     pad: c_ushort,  // U16 pad : 16;
// }
//
// impl ZCanHdrInfo {
//     /// It may result in unexpected errors that setting value out of range.
//     /// ZCanFrameInfoField::TxMode 0~15
//     /// ZCanFrameInfoField::FrameType 0~15
//     /// Others: 0~1
//     #[inline(always)]
//     pub fn set_field(&mut self, field: ZCanHdrInfoField, value: u8) -> &mut Self {
//         let value = value as u32;
//         match field {
//             ZCanHdrInfoField::TxMode => self.mode = (self.mode & 0xF0) | ((value & 0x0F) as u8), // self.mode = (self.mode & 0xF0) | ((value & 0x0F) as u8) << 0,
//             ZCanHdrInfoField::FrameType => self.mode = (self.mode & 0x0F) | ((value & 0x0F) as u8) << 4,
//             ZCanHdrInfoField::IsRemoteFrame => self.flag = (self.flag & (0xFF - 1)) | ((value & 0x01) as u8), // self.flag = (self.flag & (0xFE)) | ((value & 0x01) as u8) << 0,
//             ZCanHdrInfoField::IsExtendFrame => self.flag = (self.flag & (0xFF - (1 << 1))) | ((value & 0x01) as u8) << 1,
//             ZCanHdrInfoField::IsErrorFrame => self.flag = (self.flag & (0xFF - (1 << 2))) | ((value & 0x01) as u8) << 2,
//             ZCanHdrInfoField::IsBitrateSwitch => self.flag = (self.flag & (0xFF - (1 << 3))) | ((value & 0x01) as u8) << 3,
//             ZCanHdrInfoField::IsErrorStateIndicator => self.flag = (self.flag & (0xFF - (1 << 4))) | ((value & 0x01) as u8) << 4,
//         }
//         self
//     }
//
//     #[inline(always)]
//     pub fn get_field(&self, field: ZCanHdrInfoField) -> u8 {
//         match field {
//             ZCanHdrInfoField::TxMode => self.mode & 0x0F,     //(self.mode & 0x0F) >> 0,
//             ZCanHdrInfoField::FrameType => (self.mode & 0xF0) >> 4,
//             ZCanHdrInfoField::IsRemoteFrame => self.flag & (1 << 0),   // (self.flag & (1 << 0)) >> 0,
//             ZCanHdrInfoField::IsExtendFrame => (self.flag & (1 << 1)) >> 1,
//             ZCanHdrInfoField::IsErrorFrame => (self.flag & (1 << 2)) >> 2,
//             ZCanHdrInfoField::IsBitrateSwitch => (self.flag & (1 << 3)) >> 3,
//             ZCanHdrInfoField::IsErrorStateIndicator => (self.flag & (1 << 4)) >> 4,
//         }
//     }
//
//     #[inline(always)]
//     pub fn value(&self) -> u32 {
//         ((self.mode as u32) << 24) |
//             ((self.flag as u32) << 16) |
//             self.pad as u32
//     }
// }
//
// #[inline]
// pub(crate) fn set_extend(info: &mut ZCanHdrInfo, can_id: u32) {
//     if (can_id & IdentifierFlags::EXTENDED.bits()) > 0 {
//         info.set_field(ZCanHdrInfoField::IsExtendFrame, 1);
//     }
// }
//
// #[inline]
// pub(crate) fn set_remote(info: &mut ZCanHdrInfo, can_id: u32) {
//     if (can_id & IdentifierFlags::REMOTE.bits()) > 0 {
//         info.set_field(ZCanHdrInfoField::IsRemoteFrame, 1);
//     }
// }
//
// #[inline]
// pub(crate) fn set_error(info: &mut ZCanHdrInfo, can_id: u32) {
//     if (can_id & IdentifierFlags::ERROR.bits()) > 0 {
//         info.set_field(ZCanHdrInfoField::IsErrorFrame, 1);
//     }
// }
//
// #[cfg(test)]
// mod tests {
//     use super::{ZCanFrameType, ZCanHdrInfo, ZCanHdrInfoField, ZCanTxMode};
//
//     #[test]
//     fn frame_info() {
//         let info: ZCanHdrInfo = Default::default();
//         assert_eq!(info.mode, 0);
//         assert_eq!(info.flag, 0);
//         assert_eq!(info.pad, 0);
//
//         let mut info: ZCanHdrInfo = Default::default();
//         info.set_field(ZCanHdrInfoField::TxMode, ZCanTxMode::Normal as u8);
//         assert_eq!(info.mode, 0);
//         assert_eq!(info.flag, 0);
//         assert_eq!(info.pad, 0);
//         info.set_field(ZCanHdrInfoField::TxMode, ZCanTxMode::Once as u8);
//         assert_eq!(info.mode, 1);
//         assert_eq!(info.flag, 0);
//         assert_eq!(info.pad, 0);
//         info.set_field(ZCanHdrInfoField::TxMode, ZCanTxMode::SelfReception as u8);
//         assert_eq!(info.mode, 2);
//         assert_eq!(info.flag, 0);
//         assert_eq!(info.pad, 0);
//         info.set_field(ZCanHdrInfoField::TxMode, ZCanTxMode::SelfReceptionOnce as u8);
//         assert_eq!(info.mode, 3);
//         assert_eq!(info.flag, 0);
//         assert_eq!(info.pad, 0);
//
//         let mut info: ZCanHdrInfo = Default::default();
//         info.set_field(ZCanHdrInfoField::FrameType, ZCanFrameType::CAN as u8);
//         assert_eq!(info.mode, 0x0);
//         assert_eq!(info.flag, 0);
//         assert_eq!(info.pad, 0);
//         info.set_field(ZCanHdrInfoField::FrameType, ZCanFrameType::CANFD as u8);
//         assert_eq!(info.mode, 0x10);
//         assert_eq!(info.flag, 0);
//         assert_eq!(info.pad, 0);
//
//         let mut info: ZCanHdrInfo = Default::default();
//         info.set_field(ZCanHdrInfoField::IsRemoteFrame, 0);
//         assert_eq!(info.mode, 0x0);
//         assert_eq!(info.flag, 0);
//         assert_eq!(info.pad, 0);
//         info.set_field(ZCanHdrInfoField::IsRemoteFrame, 1);
//         assert_eq!(info.mode, 0x0);
//         assert_eq!(info.flag, 0x01);
//         assert_eq!(info.pad, 0);
//
//         let mut info: ZCanHdrInfo = Default::default();
//         info.set_field(ZCanHdrInfoField::IsExtendFrame, 0);
//         assert_eq!(info.mode, 0x0);
//         assert_eq!(info.flag, 0);
//         assert_eq!(info.pad, 0);
//         info.set_field(ZCanHdrInfoField::IsExtendFrame, 1);
//         assert_eq!(info.mode, 0x0);
//         assert_eq!(info.flag, 0x02);
//         assert_eq!(info.pad, 0);
//
//         let mut info: ZCanHdrInfo = Default::default();
//         info.set_field(ZCanHdrInfoField::IsErrorFrame, 0);
//         assert_eq!(info.mode, 0x0);
//         assert_eq!(info.flag, 0);
//         assert_eq!(info.pad, 0);
//         info.set_field(ZCanHdrInfoField::IsErrorFrame, 1);
//         assert_eq!(info.mode, 0x0);
//         assert_eq!(info.flag, 0x04);
//         assert_eq!(info.pad, 0);
//
//         let mut info: ZCanHdrInfo = Default::default();
//         info.set_field(ZCanHdrInfoField::IsBitrateSwitch, 0);
//         assert_eq!(info.mode, 0x0);
//         assert_eq!(info.flag, 0);
//         assert_eq!(info.pad, 0);
//         info.set_field(ZCanHdrInfoField::IsBitrateSwitch, 1);
//         assert_eq!(info.mode, 0x0);
//         assert_eq!(info.flag, 0x08);
//         assert_eq!(info.pad, 0);
//
//         let mut info: ZCanHdrInfo = Default::default();
//         info.set_field(ZCanHdrInfoField::IsErrorStateIndicator, 0);
//         assert_eq!(info.mode, 0x0);
//         assert_eq!(info.flag, 0);
//         assert_eq!(info.pad, 0);
//         info.set_field(ZCanHdrInfoField::IsErrorStateIndicator, 1);
//         assert_eq!(info.mode, 0x0);
//         assert_eq!(info.flag, 0x10);
//         assert_eq!(info.pad, 0);
//     }
// }

