use std::collections::HashMap;
use std::ffi::{c_uchar, c_uint, c_ushort};
use rs_can::CanError;
use crate::can::frame::ZCanHeaderV1;
use super::constant::{BRP, CANERR_FRAME_LENGTH, SJW, SMP, TSEG1, TSEG2, ZCanChlMode, ZCanChlType, ZCanFilterType};

/// Linux USBCAN USBCAN_4E(8_E) USBCANFD_800U and windows
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ZCanChlCfg {
    acc_code: c_uint,
    acc_mask: c_uint,
    #[allow(dead_code)]
    reserved: c_uint,
    filter: c_uchar,
    timing0: c_uchar,
    timing1: c_uchar,
    mode: c_uchar,
}
impl ZCanChlCfg {
    #[inline(always)]
    pub fn new(
        mode: u8,
        timing0: u32,
        timing1: u32,
        filter: u8,
        acc_code: Option<u32>,
        acc_mask: Option<u32>
    ) -> Result<Self, CanError> {
        let mode = ZCanChlMode::try_from(mode)?;
        let filter = ZCanFilterType::try_from(filter)?;
        Ok(Self {
            acc_code: acc_code.unwrap_or(0),
            acc_mask: acc_mask.unwrap_or(0xFFFFFFFF),
            reserved: Default::default(),
            filter: filter as u8,
            timing0: timing0 as u8,
            timing1: timing1 as u8,
            mode: mode as u8,
        })
    }
}

/// Linux USBCAN_4E_8E USBCANFD_800U and windows
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ZCanFdChlCfgV1 {
    acc_code: c_uint,
    acc_mask: c_uint,
    timing0: c_uint,    // abit_timing when USBCANFD
    timing1: c_uint,    // dbit_timing when USBCANFD
    brp: c_uint,
    filter: c_uchar,
    mode: c_uchar,
    #[allow(dead_code)]
    pad: c_ushort,
    #[allow(dead_code)]
    reserved: c_uint,
}
impl ZCanFdChlCfgV1 {
    #[inline(always)]
    pub fn new(
        mode: u8,
        timing0: u32,
        timing1: u32,
        filter: u8,
        acc_code: Option<u32>,
        acc_mask: Option<u32>,
        brp: Option<u32>
    ) -> Result<Self, CanError> {
        let mode = ZCanChlMode::try_from(mode)?;
        let filter = ZCanFilterType::try_from(filter)?;
        Ok(Self {
            acc_code: acc_code.unwrap_or(0),
            acc_mask: acc_mask.unwrap_or(0xFFFFFFFF),
            timing0,
            timing1,
            brp: brp.unwrap_or(0),
            filter: filter as u8,
            mode: mode as u8,
            pad: Default::default(),
            reserved: Default::default(),
        })
    }
}

/// Linux USBCANFD
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ZCanFdChlCfgSet {
    tseg1: c_uchar,
    tseg2: c_uchar,
    sjw: c_uchar,
    smp: c_uchar,
    brp: c_ushort,
}

impl TryFrom<&HashMap<String, u32>> for ZCanFdChlCfgSet {
    type Error = CanError;
    fn try_from(value: &HashMap<String, u32>) -> Result<Self, Self::Error> {
        let tseg1 = value.get(TSEG1)
            .ok_or(CanError::DeviceConfigError(format!("`{}` is not configured in file!", TSEG1)))?;
        let tseg2 = value.get(TSEG2)
            .ok_or(CanError::DeviceConfigError(format!("ZLGCAN - `{}` is not configured in file!", TSEG2)))?;
        let sjw = value.get(SJW)
            .ok_or(CanError::DeviceConfigError(format!("ZLGCAN - `{}` is not configured in file!", SJW)))?;
        let smp = value.get(SMP)
            .ok_or(CanError::DeviceConfigError(format!("ZLGCAN - `{}` is not configured in file!", SMP)))?;
        let brp = value.get(BRP)
            .ok_or(CanError::DeviceConfigError(format!("ZLGCAN - `{}` is not configured in file!", BRP)))?;

        Ok(Self::new(*tseg1, *tseg2, *sjw, *smp, *brp))
    }
}

impl ZCanFdChlCfgSet {
    #[inline(always)]
    pub fn new(tseg1: u32, tseg2: u32, sjw: u32, smp: u32, brp: u32) -> Self {
        Self {
            tseg1: tseg1 as u8,
            tseg2: tseg2 as u8,
            sjw: sjw as u8,
            smp: smp as u8,
            brp: brp as u16,
        }
    }
    /// Only used for USBCANFD-800U
    #[inline(always)]
    pub fn get_timing(&self) -> u32 {
        (self.brp as u32) << 22
            | (self.sjw as u32 & 0x7f) << 15
            | (self.tseg2 as u32 & 0x7f) << 8
            | (self.tseg1 as u32)
    }
}
/// Linux USBCANFD
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ZCanFdChlCfgV2 {
    #[doc = "< clock(Hz)"]
    clk: c_uint,
    #[doc = "< bit0-normal/listen_only, bit1-ISO/BOSCH"]
    mode: c_uint,
    aset: ZCanFdChlCfgSet,
    dset: ZCanFdChlCfgSet,
}
impl ZCanFdChlCfgV2 {
    #[inline(always)]
    pub fn new(
        can_type: u8,
        mode: u8,
        clock: u32,
        aset: ZCanFdChlCfgSet,
        dset: ZCanFdChlCfgSet
    ) -> Result<Self, CanError> {
        let can_type = ZCanChlType::try_from(can_type)?;
        let mode = ZCanChlMode::try_from(mode)?;
        let mut mode = mode as u32;
        if let ZCanChlType::CANFD_NON_ISO = can_type {
            mode |= 2;
        }
        Ok(Self {
            clk: clock,
            mode,
            aset,
            dset,
        })
    }
}
/// end of Linux USBCANFD

#[repr(C)]
pub union ZCanChlCfgV1Union {
    can: ZCanChlCfg,
    canfd: ZCanFdChlCfgV1,
}
impl From<ZCanChlCfg> for ZCanChlCfgV1Union {
    fn from(value: ZCanChlCfg) -> Self {
        Self { can: value }
    }
}
impl From<ZCanFdChlCfgV1> for ZCanChlCfgV1Union {
    fn from(value: ZCanFdChlCfgV1) -> Self {
        Self { canfd: value }
    }
}

/// CAN channel configuration v1.
/// used windows and USBCAN_4E(8_E) or USBCANFD-800U on Linux
#[repr(C)]
pub struct ZCanChlCfgV1 {
    can_type: c_uint,
    cfg: ZCanChlCfgV1Union,
}
impl ZCanChlCfgV1 {
    #[inline(always)]
    pub fn new(
        can_type: u8,
        cfg: ZCanChlCfgV1Union
    ) -> Result<Self, CanError> {
        let can_type = ZCanChlType::try_from(can_type)?;
        let can_type = match can_type {
            ZCanChlType::CAN | ZCanChlType::CANFD_ISO => ZCanChlType::CANFD_ISO,
            v => v,
        };
        Ok(Self { can_type: can_type as u32, cfg })
    }
}

/// CAN channel configuration v2.
/// used USBCAN USBCANFD on Linux
#[repr(C)]
pub union ZCanChlCfgV2 {
    can: ZCanChlCfg,
    canfd: ZCanFdChlCfgV2
}

impl From<ZCanChlCfg> for ZCanChlCfgV2 {
    fn from(value: ZCanChlCfg) -> Self {
        Self { can: value }
    }
}

impl From<ZCanFdChlCfgV2> for ZCanChlCfgV2 {
    fn from(value: ZCanFdChlCfgV2) -> Self {
        Self { canfd: value }
    }
}

#[allow(non_snake_case)]
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct ZCanChlStatus {
    pub errInterrupt: c_uchar,  /**< not used(for backward compatibility) */
    pub regMode: c_uchar,       /**< not used */
    pub regStatus: c_uchar,     /**< not used */
    pub regALCapture: c_uchar,  /**< not used */
    pub regECCapture: c_uchar,  /**< not used */
    pub regEWLimit: c_uchar,    /**< not used */
    pub regRECounter: c_uchar,  /**< RX errors */
    pub regTECounter: c_uchar,  /**< TX errors */
    pub Reserved: c_uint,
}

/// used by USBCANFD on linux
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct ZCanChlErrorV1 {
    pub hdr: ZCanHeaderV1,
    pub data: [c_uchar; CANERR_FRAME_LENGTH],
}

/// USBCAN USBCAN-4_E/8_E USBCAN-800U in linux and windows
#[allow(non_snake_case)]
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct ZCanChlErrorV2 {
    pub error_code: c_uint,
    pub passive_ErrData: [c_uchar; 3usize],
    pub arLost_ErrData: c_uchar,
}

#[repr(C)]
pub union ZCanChlError {
    v1: ZCanChlErrorV1,
    v2: ZCanChlErrorV2,
}

impl From<ZCanChlErrorV1> for ZCanChlError {
    fn from(value: ZCanChlErrorV1) -> Self {
        Self { v1: value }
    }
}

impl From<&ZCanChlError> for ZCanChlErrorV1 {
    fn from(value: &ZCanChlError) -> Self {
        unsafe { value.v1 }
    }
}

impl From<ZCanChlErrorV2> for ZCanChlError {
    fn from(value: ZCanChlErrorV2) -> Self {
        Self { v2: value }
    }
}

impl From<&ZCanChlError> for ZCanChlErrorV2 {
    fn from(value: &ZCanChlError) -> Self {
        unsafe { value.v2 }
    }
}
