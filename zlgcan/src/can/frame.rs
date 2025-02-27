use std::ffi::{c_uchar, c_uint, c_ushort};
use rs_can::{CAN_FRAME_MAX_SIZE, CanError, IdentifierFlags, SFF_MASK, EFF_MASK, CANFD_FRAME_MAX_SIZE};
use rs_can::utils::data_resize;
use crate::can::TIME_FLAG_VALID;
use super::constant::{ZCanHdrInfoField, CANFD_BRS, CANFD_ESI};

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

pub trait NewZCanFrame {
    type Error;
    fn new<T>(
        can_id: u32,
        channel: u8,
        data: T,
        info: ZCanHdrInfo,
        timestamp: u64,
    ) -> Result<Self, Self::Error>
        where
            T: AsRef<[u8]>,
            Self: Sized;
}

/// used by usbcan
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct ZCanFrameV1 {
    pub(crate) can_id: c_uint,
    pub(crate) timestamp: c_uint,
    pub(crate) time_flag: c_uchar,     // 0 - timestamp not valuable; 1 timestamp valuable
    pub(crate) send_type: c_uchar,
    pub(crate) rem_flag: c_uchar,      //是否是远程帧
    pub(crate) ext_flag: c_uchar,      //是否是扩展帧
    pub(crate) len: c_uchar,
    pub(crate) data: [c_uchar; CAN_FRAME_MAX_SIZE],
    pub(crate) channel: c_uchar,
    #[allow(dead_code)]
    pub(crate) reserved: [c_uchar; 2],
}

impl NewZCanFrame for ZCanFrameV1 {
    type Error = CanError;
    fn new<T>(can_id: u32, channel: u8, data: T, info: ZCanHdrInfo, timestamp: u64) -> Result<Self, Self::Error>
        where
            T: AsRef<[u8]> {
        zcan_frame_new(can_id, channel, data, info, |id, _chl, data, len, info| {
            let data: [c_uchar; CAN_FRAME_MAX_SIZE] = data.try_into()
                .map_err(|_| CanError::FrameConvertFailed("data length is invalid".to_owned()))?;
            Ok(Self {
                can_id: id,
                timestamp: timestamp as u32,
                time_flag: TIME_FLAG_VALID,
                send_type: info.get_field(ZCanHdrInfoField::TxMode),
                rem_flag: info.get_field(ZCanHdrInfoField::IsRemoteFrame),
                ext_flag: info.get_field(ZCanHdrInfoField::IsExtendFrame),
                len,
                data,
                channel,
                reserved: Default::default(),
            })
        })
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct ZCanHdrInfo {
    mode: c_uchar,  // U8 txm : 4; /**< TX-mode, @see ZCAN_TX_MODE */
                    // U8 fmt : 4; /**< 0-CAN2.0, 1-CANFD */
    flag: c_uchar,  // U8 sdf : 1; /**< 0-data_frame, 1-remote_frame */
                    // U8 sef : 1; /**< 0-std_frame, 1-ext_frame */
                    // U8 err : 1; /**< error flag */
                    // U8 brs : 1; /**< bit-rate switch */
                    // U8 est : 1; /**< error state */
                    // 5~7bit not used
    #[allow(dead_code)]
    pad: c_ushort,  // U16 pad : 16;
}

impl ZCanHdrInfo {
    /// It may result in unexpected errors that setting value out of range.
    /// ZCanFrameInfoField::TxMode 0~15
    /// ZCanFrameInfoField::FrameType 0~15
    /// Others: 0~1
    #[inline(always)]
    pub(crate) fn set_field(&mut self, field: ZCanHdrInfoField, value: u8) -> &mut Self {
        let value = value as u32;
        match field {
            ZCanHdrInfoField::TxMode => self.mode = (self.mode & 0xF0) | ((value & 0x0F) as u8), // self.mode = (self.mode & 0xF0) | ((value & 0x0F) as u8) << 0,
            ZCanHdrInfoField::FrameType => self.mode = (self.mode & 0x0F) | ((value & 0x0F) as u8) << 4,
            ZCanHdrInfoField::IsRemoteFrame => self.flag = (self.flag & (0xFF - 1)) | ((value & 0x01) as u8), // self.flag = (self.flag & (0xFE)) | ((value & 0x01) as u8) << 0,
            ZCanHdrInfoField::IsExtendFrame => self.flag = (self.flag & (0xFF - (1 << 1))) | ((value & 0x01) as u8) << 1,
            ZCanHdrInfoField::IsErrorFrame => self.flag = (self.flag & (0xFF - (1 << 2))) | ((value & 0x01) as u8) << 2,
            ZCanHdrInfoField::IsBitrateSwitch => self.flag = (self.flag & (0xFF - (1 << 3))) | ((value & 0x01) as u8) << 3,
            ZCanHdrInfoField::IsErrorStateIndicator => self.flag = (self.flag & (0xFF - (1 << 4))) | ((value & 0x01) as u8) << 4,
        }
        self
    }

    #[inline(always)]
    pub(crate) fn get_field(&self, field: ZCanHdrInfoField) -> u8 {
        match field {
            ZCanHdrInfoField::TxMode => self.mode & 0x0F,     //(self.mode & 0x0F) >> 0,
            ZCanHdrInfoField::FrameType => (self.mode & 0xF0) >> 4,
            ZCanHdrInfoField::IsRemoteFrame => self.flag & (1 << 0),   // (self.flag & (1 << 0)) >> 0,
            ZCanHdrInfoField::IsExtendFrame => (self.flag & (1 << 1)) >> 1,
            ZCanHdrInfoField::IsErrorFrame => (self.flag & (1 << 2)) >> 2,
            ZCanHdrInfoField::IsBitrateSwitch => (self.flag & (1 << 3)) >> 3,
            ZCanHdrInfoField::IsErrorStateIndicator => (self.flag & (1 << 4)) >> 4,
        }
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct ZCanHeaderV1 {
    pub(crate) timestamp: c_uint,  //**< timestamp */
    pub(crate) can_id: c_uint,     //**< CAN-ID */
    pub(crate) info: ZCanHdrInfo,  //**< @see ZCAN_MSG_INF */
    #[allow(dead_code)]
    pub(crate) pad: c_ushort,
    pub(crate) channel: c_uchar,   //**< channel */
    pub(crate) len: c_uchar,       //**< data length */
}

/// used by linux
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct ZCanFrameV2 {
    pub(crate) hdr: ZCanHeaderV1,
    pub(crate) data: [c_uchar; CAN_FRAME_MAX_SIZE],
}

impl NewZCanFrame for ZCanFrameV2 {
    type Error = CanError;
    fn new<T>(can_id: u32, channel: u8, data: T, info: ZCanHdrInfo, timestamp: u64) -> Result<Self, Self::Error>
        where
            T: AsRef<[u8]> {
        zcan_frame_new(can_id, channel, data, info, |id, chl, data, len, info| {
            let data: [c_uchar; CAN_FRAME_MAX_SIZE] = data.try_into()
                .map_err(|_| CanError::FrameConvertFailed("data length is invalid".to_owned()))?;
            Ok(Self {
                hdr: ZCanHeaderV1 {
                    timestamp: timestamp as u32,
                    can_id: id,
                    info,
                    pad: Default::default(),
                    channel: chl,
                    len,
                },
                data,
            })
        })
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct ZCanHeaderV2 {
    pub(crate) can_id: c_uint,
    pub(crate) can_len: c_uchar,
    pub(crate) flag: c_uchar,
    #[allow(dead_code)]
    pub(crate) __res0: c_uchar,  /* reserved / padding used for channel */
    #[allow(dead_code)]
    pub(crate) __res1: c_uchar,  /* reserved / padding */
}

impl ZCanHeaderV2 {
    #[inline]
    pub fn update_channel(&mut self, channel: u8) {
        self.__res0 = channel;
    }
}

/// used by usbcanfd-800u usbcan-4-E usbcan-8-E and windows
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct ZCanFrameV3 {
    pub(crate) hdr: ZCanHeaderV2,
    pub(crate) data: [c_uchar; CAN_FRAME_MAX_SIZE],
    pub(crate) ts_or_mode: c_uint,       // timestamp when received
}

impl ZCanFrameV3 {
    #[inline]
    pub fn update_channel(&mut self, channel: u8) {
        self.hdr.update_channel(channel)
    }
}

impl NewZCanFrame for ZCanFrameV3 {
    type Error = CanError;
    #[allow(unused_variables)]
    fn new<T>(can_id: u32, channel: u8, data: T, info: ZCanHdrInfo, _: u64) -> Result<Self, Self::Error>
        where
            T: AsRef<[u8]> {
        zcan_frame_new2(can_id, channel, data, info,  |id, chl, data, len, info| {
            let data: [c_uchar; CAN_FRAME_MAX_SIZE] = data.try_into()
                .map_err(|_| CanError::FrameConvertFailed("data length is invalid".to_owned()))?;
            Ok(Self {
                hdr: ZCanHeaderV2 {
                    can_id: id,
                    can_len: len,
                    flag: Default::default(),
                    __res0: chl,
                    __res1: Default::default(),
                },
                data,
                ts_or_mode: info.get_field(ZCanHdrInfoField::TxMode) as u32,
            })
        })
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct CanFdData {
    pub(crate) data: [c_uchar; CANFD_FRAME_MAX_SIZE],
}

impl Default for CanFdData {
    fn default() -> Self {
        Self { data: [Default::default(); CANFD_FRAME_MAX_SIZE] }
    }
}

/// used by USBCANFD on linux
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct ZCanFdFrameV1 {
    pub(crate) hdr: ZCanHeaderV1,
    pub(crate) data: CanFdData,
}

impl NewZCanFrame for ZCanFdFrameV1 {
    type Error = CanError;
    fn new<T>(can_id: u32, channel: u8, data: T, info: ZCanHdrInfo, timestamp: u64) -> Result<Self, Self::Error>
        where
            T: AsRef<[u8]> {
        zcanfd_frame_new(can_id, channel, data, info, |id, chl, data, len, info| {
            let data: [c_uchar; CANFD_FRAME_MAX_SIZE] = data.try_into()
                .map_err(|_| CanError::FrameConvertFailed("data length is invalid".to_owned()))?;
            Ok(Self {
                hdr: ZCanHeaderV1 {
                    timestamp: timestamp as u32,
                    can_id: id,
                    info,
                    pad: Default::default(),
                    channel: chl,
                    len,
                },
                data: CanFdData { data },
            })
        })
    }
}

/// used by windows and USBCAN-4E|8E USBCANFD-800U
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct ZCanFdFrameV2 {
    pub(crate) hdr: ZCanHeaderV2,
    pub(crate) data: CanFdData,
    pub(crate) ts_or_mode: c_uint,       // timestamp when received
}

impl ZCanFdFrameV2 {
    #[inline]
    pub fn update_channel(&mut self, channel: u8) {
        self.hdr.update_channel(channel)
    }
}

impl NewZCanFrame for ZCanFdFrameV2 {
    type Error = CanError;
    #[allow(unused_variables)]
    fn new<T>(can_id: u32, channel: u8, data: T, info: ZCanHdrInfo, _: u64) -> Result<Self, Self::Error>
        where
            T: AsRef<[u8]> {
        zcanfd_frame_new2(can_id, channel, data, info, |id, chl, data, len, info| {
            let mut flag: u8 = Default::default();
            if info.get_field(ZCanHdrInfoField::IsBitrateSwitch) > 0 {
                flag |= CANFD_BRS;
            }
            if info.get_field(ZCanHdrInfoField::IsErrorStateIndicator) > 0 {
                flag |= CANFD_ESI;
            }
            let data: [c_uchar; CANFD_FRAME_MAX_SIZE] = data.try_into()
                .map_err(|_| CanError::FrameConvertFailed("data length is invalid".to_owned()))?;

            Ok(Self {
                hdr: ZCanHeaderV2 {
                    can_id: id,
                    can_len: len,
                    flag,
                    __res0: chl,
                    __res1: Default::default(),
                },
                data: CanFdData { data },
                ts_or_mode: info.get_field(ZCanHdrInfoField::TxMode) as u32,
            })
        })
    }
}

fn zcan_frame_new<T, R>(
    can_id: u32,
    channel: u8,
    data: T,
    mut info: ZCanHdrInfo,
    callback: impl Fn(u32, u8, Vec<u8>, u8, ZCanHdrInfo) -> Result<R, CanError>
) -> Result<R, CanError>
    where
        T: AsRef<[u8]> {
    match can_id {
        0..=EFF_MASK => {
            let mut data = Vec::from(data.as_ref());
            let len = data.len();
            match len {
                0..=CAN_FRAME_MAX_SIZE => {
                    set_extended(&mut info, can_id);
                    data_resize(&mut data, CAN_FRAME_MAX_SIZE);

                    callback(can_id, channel, data, len as u8, info)
                },
                _ => Err(CanError::OtherError("parameter not supported".to_owned())),
            }
        },
        _ => Err(CanError::OtherError("parameter not supported".to_owned())),
    }
}

fn zcanfd_frame_new<T, R>(
    can_id: u32,
    channel: u8,
    data: T,
    mut info: ZCanHdrInfo,
    callback: impl Fn(u32, u8, Vec<u8>, u8, ZCanHdrInfo) -> Result<R, CanError>
) -> Result<R, CanError>
    where
        T: AsRef<[u8]> {
    if let 0..=EFF_MASK = can_id {
        let mut data = Vec::from(data.as_ref());
        let len = data.len();
        if let ..=CANFD_FRAME_MAX_SIZE = len {
            set_extended(&mut info, can_id);
            data_resize(&mut data, CANFD_FRAME_MAX_SIZE);

            callback(can_id, channel, data, len as u8, info)
        }
        else {
            Err(CanError::OtherError("parameter not supported".to_owned()))
        }
    } else {
        Err(CanError::OtherError("parameter not supported".to_owned()))
    }
}

// pub(self) fn zcan_frame_new2<const MAX_LEN: usize, T, R>(can_id: u32, channel: u8, data: T, mut info: ZCanHdrInfo,
//                                    callback: impl Fn(u32, u8, Vec<u8>, u8, ZCanHdrInfo) -> R) -> Option<R>
fn zcan_frame_new2<T, R>(
    can_id: u32,
    channel: u8,
    data: T,
    mut info: ZCanHdrInfo,
    callback: impl Fn(u32, u8, Vec<u8>, u8, ZCanHdrInfo) -> Result<R, CanError>
) -> Result<R, CanError>
    where
        T: AsRef<[u8]> {
    match can_id {
        0..=EFF_MASK => {
            let mut data = Vec::from(data.as_ref());
            let len = data.len();
            match len {
                0..=CAN_FRAME_MAX_SIZE => {
                    data_resize(&mut data, CAN_FRAME_MAX_SIZE);
                    set_extended(&mut info, can_id);

                    let mut can_id = can_id;
                    if info.get_field(ZCanHdrInfoField::IsExtendFrame) > 0 {
                        can_id |= IdentifierFlags::EXTENDED.bits();
                    }
                    if info.get_field(ZCanHdrInfoField::IsRemoteFrame) > 0 {
                        can_id |= IdentifierFlags::REMOTE.bits();
                    }
                    if info.get_field(ZCanHdrInfoField::IsErrorFrame) > 0 {
                        can_id |= IdentifierFlags::ERROR.bits();
                    }
                    callback(can_id, channel, data, len as u8, info)
                },
                _ => Err(CanError::OtherError("parameter not supported".to_owned())),
            }
        },
        _ => Err(CanError::OtherError("parameter not supported".to_owned())),
    }
}

fn zcanfd_frame_new2<T, R>(
    can_id: u32,
    channel: u8,
    data: T,
    mut info: ZCanHdrInfo,
    callback: impl Fn(u32, u8, Vec<u8>, u8, ZCanHdrInfo) -> Result<R, CanError>
) -> Result<R, CanError>
    where
        T: AsRef<[u8]> {
    if let 0..=EFF_MASK = can_id {
        let mut data = Vec::from(data.as_ref());
        let len = data.len();
        if let ..=CANFD_FRAME_MAX_SIZE = len {
            data_resize(&mut data, CANFD_FRAME_MAX_SIZE);
            set_extended(&mut info, can_id);

            let mut can_id = can_id;
            if info.get_field(ZCanHdrInfoField::IsExtendFrame) > 0 {
                can_id |= IdentifierFlags::EXTENDED.bits();
            }
            if info.get_field(ZCanHdrInfoField::IsRemoteFrame) > 0 {
                can_id |= IdentifierFlags::REMOTE.bits();
            }
            if info.get_field(ZCanHdrInfoField::IsErrorFrame) > 0 {
                can_id |= IdentifierFlags::ERROR.bits();
            }
            callback(can_id, channel, data, len as u8, info)
        } else {
            Err(CanError::OtherError("parameter not supported".to_owned()))
        }
    }
    else {
        Err(CanError::OtherError("parameter not supported".to_owned()))
    }
}

#[inline]
fn set_extended(info: &mut ZCanHdrInfo, can_id: u32) {
    if (can_id & !SFF_MASK) > 0 {
        info.set_field(ZCanHdrInfoField::IsExtendFrame, 1);
    }
}

#[cfg(test)]
mod tests {
    use crate::can::constant::{ZCanFrameType, ZCanTxMode};
    use super::{ZCanHdrInfo, ZCanHdrInfoField};

    #[test]
    fn frame_info() {
        let info: ZCanHdrInfo = Default::default();
        assert_eq!(info.mode, 0);
        assert_eq!(info.flag, 0);
        assert_eq!(info.pad, 0);

        let mut info: ZCanHdrInfo = Default::default();
        info.set_field(ZCanHdrInfoField::TxMode, ZCanTxMode::Normal as u8);
        assert_eq!(info.mode, 0);
        assert_eq!(info.flag, 0);
        assert_eq!(info.pad, 0);
        info.set_field(ZCanHdrInfoField::TxMode, ZCanTxMode::Once as u8);
        assert_eq!(info.mode, 1);
        assert_eq!(info.flag, 0);
        assert_eq!(info.pad, 0);
        info.set_field(ZCanHdrInfoField::TxMode, ZCanTxMode::SelfReception as u8);
        assert_eq!(info.mode, 2);
        assert_eq!(info.flag, 0);
        assert_eq!(info.pad, 0);
        info.set_field(ZCanHdrInfoField::TxMode, ZCanTxMode::SelfReceptionOnce as u8);
        assert_eq!(info.mode, 3);
        assert_eq!(info.flag, 0);
        assert_eq!(info.pad, 0);

        let mut info: ZCanHdrInfo = Default::default();
        info.set_field(ZCanHdrInfoField::FrameType, ZCanFrameType::CAN as u8);
        assert_eq!(info.mode, 0x0);
        assert_eq!(info.flag, 0);
        assert_eq!(info.pad, 0);
        info.set_field(ZCanHdrInfoField::FrameType, ZCanFrameType::CANFD as u8);
        assert_eq!(info.mode, 0x10);
        assert_eq!(info.flag, 0);
        assert_eq!(info.pad, 0);

        let mut info: ZCanHdrInfo = Default::default();
        info.set_field(ZCanHdrInfoField::IsRemoteFrame, 0);
        assert_eq!(info.mode, 0x0);
        assert_eq!(info.flag, 0);
        assert_eq!(info.pad, 0);
        info.set_field(ZCanHdrInfoField::IsRemoteFrame, 1);
        assert_eq!(info.mode, 0x0);
        assert_eq!(info.flag, 0x01);
        assert_eq!(info.pad, 0);

        let mut info: ZCanHdrInfo = Default::default();
        info.set_field(ZCanHdrInfoField::IsExtendFrame, 0);
        assert_eq!(info.mode, 0x0);
        assert_eq!(info.flag, 0);
        assert_eq!(info.pad, 0);
        info.set_field(ZCanHdrInfoField::IsExtendFrame, 1);
        assert_eq!(info.mode, 0x0);
        assert_eq!(info.flag, 0x02);
        assert_eq!(info.pad, 0);

        let mut info: ZCanHdrInfo = Default::default();
        info.set_field(ZCanHdrInfoField::IsErrorFrame, 0);
        assert_eq!(info.mode, 0x0);
        assert_eq!(info.flag, 0);
        assert_eq!(info.pad, 0);
        info.set_field(ZCanHdrInfoField::IsErrorFrame, 1);
        assert_eq!(info.mode, 0x0);
        assert_eq!(info.flag, 0x04);
        assert_eq!(info.pad, 0);

        let mut info: ZCanHdrInfo = Default::default();
        info.set_field(ZCanHdrInfoField::IsBitrateSwitch, 0);
        assert_eq!(info.mode, 0x0);
        assert_eq!(info.flag, 0);
        assert_eq!(info.pad, 0);
        info.set_field(ZCanHdrInfoField::IsBitrateSwitch, 1);
        assert_eq!(info.mode, 0x0);
        assert_eq!(info.flag, 0x08);
        assert_eq!(info.pad, 0);

        let mut info: ZCanHdrInfo = Default::default();
        info.set_field(ZCanHdrInfoField::IsErrorStateIndicator, 0);
        assert_eq!(info.mode, 0x0);
        assert_eq!(info.flag, 0);
        assert_eq!(info.pad, 0);
        info.set_field(ZCanHdrInfoField::IsErrorStateIndicator, 1);
        assert_eq!(info.mode, 0x0);
        assert_eq!(info.flag, 0x10);
        assert_eq!(info.pad, 0);
    }
}

