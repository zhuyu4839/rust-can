use std::collections::HashMap;
use std::ffi::{c_uchar, c_uint, c_ushort};
use rs_can::CanError;
use crate::can::{common::BitrateCfg, CanChlCfgExt, ZCanChlMode, ZCanChlType, constant::{BRP, SJW, SMP, TSEG1, TSEG2}};

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
            .ok_or(CanError::OtherError(format!("`{}` is not configured in file!", TSEG1)))?;
        let tseg2 = value.get(TSEG2)
            .ok_or(CanError::OtherError(format!("ZLGCAN - `{}` is not configured in file!", TSEG2)))?;
        let sjw = value.get(SJW)
            .ok_or(CanError::OtherError(format!("ZLGCAN - `{}` is not configured in file!", SJW)))?;
        let smp = value.get(SMP)
            .ok_or(CanError::OtherError(format!("ZLGCAN - `{}` is not configured in file!", SMP)))?;
        let brp = value.get(BRP)
            .ok_or(CanError::OtherError(format!("ZLGCAN - `{}` is not configured in file!", BRP)))?;

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
pub(crate) struct ZCanFdChlCfgInner {
    #[doc = "< clock(Hz)"]
    clk: c_uint,
    #[doc = "< bit0-normal/listen_only, bit1-ISO/BOSCH"]
    mode: c_uint,
    aset: ZCanFdChlCfgSet,
    dset: ZCanFdChlCfgSet,
}
impl ZCanFdChlCfgInner {
    #[inline(always)]
    pub fn new(
        can_type: ZCanChlType,
        mode: ZCanChlMode,
        clock: u32,
        aset: ZCanFdChlCfgSet,
        dset: ZCanFdChlCfgSet
    ) -> Self {
        let mut mode = mode as u32;
        if let ZCanChlType::CANFD_NON_ISO = can_type {
            mode |= 2;
        }
        Self {
            clk: clock,
            mode,
            aset,
            dset,
        }
    }
}
/// end of Linux USBCANFD

// #[repr(C)]
// pub union ZCanFdChlCfgUnion {
//     // USBCANFD
//     pub(crate) v1: self::ZCanFdChlCfgInner,
//     // USBCAN_4E_8E USBCANFD_800U
//     pub(crate) v2: super::common::ZCanFdChlCfgInner,
// }

#[repr(C)]
#[derive(Copy, Clone)]
pub union ZCanChlCfgUnion {
    pub(crate) can: super::common::ZCanChlCfgInner,
    pub(crate) canfd: super::common::ZCanFdChlCfgInner,
}

pub(crate) fn get_fd_cfg(
    can_type: u8,
    mode: u8,
    bitrate: u32,
    cfg_ctx: &BitrateCfg,
    extra: &CanChlCfgExt,
) -> Result<self::ZCanFdChlCfgInner, CanError> {
    let (aset, dset) = get_fd_set(bitrate, cfg_ctx, extra.dbitrate)?;
    let clock = cfg_ctx.clock
        .ok_or(CanError::OtherError("`clock` is not configured in file!".to_string()))?;
    let can_type = ZCanChlType::try_from(can_type)?;

    Ok(self::ZCanFdChlCfgInner::new(
        can_type,
        ZCanChlMode::try_from(mode)?,
        clock,
        aset,
        dset,
    ))
    // match dev_type {
    //     ZCanDeviceType::ZCAN_USBCAN_4E_U |
    //     ZCanDeviceType::ZCAN_USBCANFD_800U => {
    //
    //     },
    //     _ => {
    //         let timing0 = aset.get_timing();    // 4458527 = 0x44081f
    //         let timing1 = dset.get_timing();    // 4260357 = 0x410205
    //         Ok(ZCanChlCfgUnion {
    //             canfd: ZCanFdChlCfgUnion {
    //                 v2: super::common::ZCanFdChlCfgInner::new(
    //                     mode,
    //                     timing0,
    //                     timing1,
    //                     extra.filter,
    //                     extra.acc_code,
    //                     extra.acc_mask,
    //                     extra.brp,
    //                 )?
    //             }
    //         })
    //     },
    // }
}

fn get_fd_set(
    bitrate: u32,
    cfg: &BitrateCfg,
    dbitrate: Option<u32>
) -> Result<(ZCanFdChlCfgSet, ZCanFdChlCfgSet), CanError> {
    let bitrate_ctx = &cfg.bitrate;
    let dbitrate_ctx = &cfg.data_bitrate;
    let aset = bitrate_ctx
        .get(&bitrate.to_string())
        .ok_or(CanError::OtherError(format!("bitrate `{}` is not configured in file!", bitrate)))?;
    let dset=
        match dbitrate {
            Some(v) => {    // dbitrate is not None
                match dbitrate_ctx {
                    Some(ctx) => {  // dbitrate context is not None
                        match ctx.get(&v.to_string()) {
                            Some(value) => Ok(value),
                            None => Err(CanError::OtherError(format!("data bitrate `{}` is not configured in file!", v))),
                        }
                    },
                    None => {   // dbitrate context is None
                        match bitrate_ctx.get(&v.to_string()) {
                            Some(value) => Ok(value),
                            None => Err(CanError::OtherError(format!("data bitrate `{}` is not configured in file!", v))),
                        }
                    }
                }
            },
            None => {   // dbitrate is None
                match dbitrate_ctx {
                    Some(ctx) => {
                        match ctx.get(&bitrate.to_string()) {
                            Some(value) => Ok(value),
                            None => Ok(aset),
                        }
                    },
                    None => Ok(aset),
                }
            }
        }?;

    Ok((ZCanFdChlCfgSet::try_from(aset)?, ZCanFdChlCfgSet::try_from(dset)?))
}
