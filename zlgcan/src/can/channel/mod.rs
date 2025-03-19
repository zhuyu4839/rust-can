pub(crate) mod common;
pub use common::{CanChlCfg, CanChlCfgExt, CanChlCfgFactory, ZCanChlStatus, ZCanChlType, ZCanChlMode};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub(crate) use linux::*;
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub(crate) use windows::*;

use std::ffi::c_uint;
use rs_can::CanError;
use crate::device::ZCanDeviceType;

#[repr(C)]
pub struct ZCanChlCfg {
    can_type: c_uint,
    cfg: ZCanChlCfgUnion,
}

impl ZCanChlCfg {
    #[inline(always)]
    pub fn new(
        dev_type: ZCanDeviceType,
        can_type: ZCanChlType,
        cfg: ZCanChlCfgUnion
    ) -> Self {
        let can_type = if dev_type.canfd_support() {
            match can_type {
                ZCanChlType::CAN | ZCanChlType::CANFD_ISO => ZCanChlType::CANFD_ISO,
                v => v,
            }
        }
        else {
            ZCanChlType::CAN
        };

        Self { can_type: can_type as u32, cfg }
    }
}

impl TryFrom<&CanChlCfg> for ZCanChlCfg {
    type Error = CanError;
    fn try_from(cfg: &CanChlCfg) -> Result<Self, Self::Error> {
        let dev_type = cfg.dev_type;
        let binding = cfg.cfg_ctx.upgrade()
            .ok_or(CanError::OtherError("Failed to upgrade configuration context".to_string()))?;
        let cfg_ctx = binding.get(&dev_type.to_string())
            .ok_or(CanError::OtherError(format!("device: {:?} is not configured in file!", dev_type)))?;
        let dev_type = cfg.device_type()?;

        let cfg = if dev_type.canfd_support() {
            let cfg = ZCanChlCfgUnion {
                canfd: common::ZCanFdChlCfgInner::new(
                    cfg.mode,
                    0,  // TODO timing0 and timing1 ignored
                    0,
                    cfg.extra.filter,
                    cfg.extra.acc_code,
                    cfg.extra.acc_mask,
                    cfg.extra.brp)?
            };

            Ok(cfg)
        }
        else {
            let bitrate = cfg.bitrate;
            Ok(ZCanChlCfgUnion {
                can: common::ZCanChlCfgInner::try_from_with(cfg_ctx, cfg.mode, bitrate, cfg.extra())?
            })
        }?;

        Ok(ZCanChlCfg::new(dev_type, ZCanChlType::CAN, cfg))
    }
}
