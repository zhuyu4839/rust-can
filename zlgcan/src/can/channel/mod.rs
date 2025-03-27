pub(crate) mod common;
pub use common::{ZCanChlStatus, ZCanChlType, ZCanChlMode};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub(crate) use linux::*;
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub(crate) use windows::*;

use std::ffi::c_uint;
use rs_can::{CanError, ChannelConfig};
use crate::can::common::BitrateCfg;
use crate::{ACC_CODE, ACC_MASK, BRP, CHANNEL_MODE, FILTER};
use crate::device::ZCanDeviceType;

#[repr(C)]
pub(crate) struct ZCanChlCfg {
    can_type: c_uint,
    cfg: ZCanChlCfgUnion,
}

impl ZCanChlCfg {
    #[inline(always)]
    pub fn new(
        dev_type: ZCanDeviceType,
        can_type: ZCanChlType,
        ctx: &BitrateCfg,
        cfg: &ChannelConfig,
    ) -> Result<Self, CanError> {
        if dev_type.canfd_support() {
            Ok(Self {
                can_type: can_type as u32,
                cfg: ZCanChlCfgUnion {
                    canfd: common::ZCanFdChlCfgInner::new(
                        cfg.get_other::<u8>(CHANNEL_MODE)?
                            .unwrap_or(ZCanChlMode::Normal as u8),
                        0,  // TODO timing0 and timing1 ignored
                        0,
                        cfg.get_other::<u8>(FILTER)?
                            .unwrap_or(0x00),
                        cfg.get_other::<u32>(ACC_CODE)?,
                        cfg.get_other::<u32>(ACC_MASK)?,
                        cfg.get_other::<u32>(BRP)?,
                    )?
                }
            })
        }
        else {
            Ok(Self {
                can_type: ZCanChlType::CAN as u32,
                cfg: ZCanChlCfgUnion {
                    can: common::ZCanChlCfgInner::try_from_with(ctx, cfg)?
                }
            })
        }
    }
}
