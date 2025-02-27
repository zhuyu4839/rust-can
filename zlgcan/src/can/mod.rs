mod channel;
mod constant;
mod frame;
mod message;
mod util;

pub use channel::*;
pub use constant::*;
pub use frame::*;
pub use message::*;

use std::{collections::HashMap, fs::read_to_string};
use std::sync::{Arc, Weak};
use serde::Deserialize;
use rs_can::CanError;
use crate::device::ZCanDeviceType;

/// The deserialize object mapped to configuration file context.
#[derive(Debug, Deserialize)]
pub struct BitrateCfg {
    pub(crate) bitrate: HashMap<String, HashMap<String, u32>>,
    pub(crate) clock: Option<u32>,
    pub(crate) data_bitrate: Option<HashMap<String, HashMap<String, u32>>>
}

impl BitrateCfg {
    #[inline]
    pub fn bitrate(&self) -> &HashMap<String, HashMap<String, u32>> {
        &self.bitrate
    }
    #[inline]
    pub fn clock(&self) -> Option<u32> {
        self.clock.clone()
    }
    #[inline]
    pub fn dbitrate(&self) -> &Option<HashMap<String, HashMap<String, u32>>> {
        &self.data_bitrate
    }
}

/// The extra info for common CAN channel configuration.
#[derive(Debug, Default, Copy, Clone)]
pub struct CanChlCfgExt {
    filter: u8,
    dbitrate: Option<u32>,
    resistance: Option<bool>,
    acc_code: Option<u32>,
    acc_mask: Option<u32>,
    brp: Option<u32>,
}

impl CanChlCfgExt {
    pub fn new(
        filter: Option<u8>,
        dbitrate: Option<u32>,
        resistance: Option<bool>,
        acc_code: Option<u32>,
        acc_mask: Option<u32>,
        brp: Option<u32>
    ) -> Self {
        Self {
            filter: filter.unwrap_or_default(),
            // canfd,
            dbitrate,
            resistance,
            acc_code,
            acc_mask,
            brp,
        }
    }
    #[inline(always)]
    pub fn filter(&self) -> Result<ZCanFilterType, CanError> {
        ZCanFilterType::try_from(self.filter)
    }
    #[inline(always)]
    pub fn dbitrate(&self) -> Option<u32> {
        self.dbitrate
    }
    #[inline(always)]
    pub fn resistance(&self) -> bool {
        self.resistance.unwrap_or(true)
    }
    #[inline(always)]
    pub fn acc_code(&self) -> u32 {
        self.acc_code.unwrap_or_default()
    }
    #[inline(always)]
    pub fn acc_mask(&self) -> u32 {
        self.acc_mask.unwrap_or(0xFFFFFFFF)
    }
    #[inline]
    pub fn brp(&self) -> u32 {
        self.brp.unwrap_or_default()
    }
}

/// The common CAN channel configuration.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct CanChlCfg {
    dev_type: u32,
    can_type: u8,
    mode: u8,
    bitrate: u32,
    extra: CanChlCfgExt,
    cfg_ctx: Weak<HashMap<String, BitrateCfg>>,
    // cfg_ctx: HashMap<String, BitrateCfg>,
}

impl CanChlCfg {
    pub fn new(
        dev_type: u32,
        can_type: u8,
        mode: u8,
        bitrate: u32,
        extra: CanChlCfgExt,
        cfg_ctx: Weak<HashMap<String, BitrateCfg>>
    ) -> Self {
        Self {
            dev_type,
            can_type,
            mode,
            bitrate,
            extra,
            cfg_ctx,
        }
    }
    #[inline(always)]
    pub fn device_type(&self) -> Result<ZCanDeviceType, CanError> {
        ZCanDeviceType::try_from(self.dev_type)
    }
    #[inline(always)]
    pub fn can_type(&self) -> Result<ZCanChlType, CanError> {
        ZCanChlType::try_from(self.can_type)
    }
    #[inline(always)]
    pub fn clock(&self) -> Option<u32> {
        if let Some(ctx) = self.cfg_ctx.upgrade() {
            if let Some(cfg) = ctx.get(&self.bitrate.to_string()) {
                return cfg.clock;
            }
        }
        None
    }
    #[inline]
    pub fn configuration(&self) -> &Weak<HashMap<String, BitrateCfg>> {
        &self.cfg_ctx
    }
    #[inline]
    pub fn mode(&self) -> u8 {
        self.mode
    }
    #[inline(always)]
    pub fn bitrate(&self) -> u32 {
        self.bitrate
    }
    #[inline(always)]
    pub fn extra(&self) -> &CanChlCfgExt {
        &self.extra
    }
}

fn to_chl_cfg(mode: u8, bitrate: u32, cfg_ctx: &BitrateCfg, ext: &CanChlCfgExt) -> Result<ZCanChlCfg, CanError> {
    match cfg_ctx.bitrate.get(&bitrate.to_string()) {
        Some(v) => {
            let timing0 = v.get(TIMING0)
                .ok_or(CanError::DeviceConfigError(format!("`{}` is not configured in file!", TIMING0)))?;
            let timing1 = v.get(TIMING1)
                .ok_or(CanError::DeviceConfigError(format!("`{}` is not configured in file!", TIMING1)))?;
            ZCanChlCfg::new(
                mode, *timing0, *timing1, ext.filter, ext.acc_code, ext.acc_mask
            )
        },
        None => Err(CanError::DeviceConfigError(
            format!("the bitrate: `{}` is not configured", bitrate)
        )),
    }
}

impl TryFrom<&CanChlCfg> for ZCanChlCfgV1 {
    type Error = CanError;

    fn try_from(value: &CanChlCfg) -> Result<Self, Self::Error> {
        let dev_type = value.dev_type;
        let binding = value.cfg_ctx.upgrade()
            .ok_or(CanError::DeviceConfigError("Failed to upgrade configuration context".to_string()))?;
        let cfg = binding.get(&dev_type.to_string())
            .ok_or(CanError::DeviceConfigError(format!("device: {:?} is not configured in file!", dev_type)))?;
        let dev_type = value.device_type()?;
        match dev_type {
            ZCanDeviceType::ZCAN_USBCANFD_800U => {
                let ext = &value.extra;
                let (aset, dset) = get_fd_set(value, cfg, ext.dbitrate)?;
                let timing0 = aset.get_timing();    // 4458527 = 0x44081f
                let timing1 = dset.get_timing();    // 4260357 = 0x410205
                return ZCanChlCfgV1::new(
                    value.can_type,
                    ZCanChlCfgV1Union::from(
                        ZCanFdChlCfgV1::new(
                            value.mode,
                            timing0,
                            timing1,
                            ext.filter, ext.acc_code, ext.acc_mask, ext.brp,
                    )?)
                );
            },
            _ => {},
        }
        if dev_type.canfd_support() {      // the device supported canfd can't set CAN type to CAN
            let ext = &value.extra;
            ZCanChlCfgV1::new(
                value.can_type,
                ZCanChlCfgV1Union::from(
                    ZCanFdChlCfgV1::new(
                        value.mode, 0, 0, ext.filter, ext.acc_code, ext.acc_mask, ext.brp  // TODO timing0 and timing1 ignored
                    )?
                )
            )
        }
        else {
            ZCanChlCfgV1::new(
                ZCanChlType::CAN as u8,
                ZCanChlCfgV1Union::from(
                    to_chl_cfg(value.mode, value.bitrate, cfg, &value.extra)?
                )
            )
        }
    }
}

impl TryFrom<&CanChlCfg> for ZCanChlCfgV2 {

    type Error = CanError;
    fn try_from(value: &CanChlCfg) -> Result<Self, Self::Error> {
        let dev_type = value.dev_type;
        let binding = value.cfg_ctx.upgrade()
            .ok_or(CanError::DeviceConfigError("Failed to upgrade configuration context".to_string()))?;
        let cfg = binding.get(&dev_type.to_string())
            .ok_or(CanError::DeviceConfigError(format!("device: {:?} is not configured in file!", dev_type)))?;
        if value.device_type()?
            .canfd_support() {
            let clock = cfg.clock
                .ok_or(CanError::DeviceConfigError("`clock` is not configured in file!".to_string()))?;
            let ext = &value.extra;
            let (aset, dset) = get_fd_set(value, cfg, ext.dbitrate)?;
            Ok(Self::from(
                ZCanFdChlCfgV2::new(
                    value.can_type,
                    value.mode,
                    clock,
                    aset,
                    dset
                )?
            ))
        }
        else {
            Ok(Self::from(
                to_chl_cfg(value.mode, value.bitrate, cfg, &value.extra)?
            ))
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct CanChlCfgFactory(Arc<HashMap<String, BitrateCfg>>);


impl CanChlCfgFactory {
    pub fn new() -> Result<Self, CanError> {
        let libpath = match dotenvy::from_filename(ZCAN_ENV) {
            Ok(_) => match std::env::var(ZCAN_VAR){
                Ok(v) => format!("{}/{}", v, BITRATE_CFG_FILENAME),
                Err(_) => BITRATE_CFG_FILENAME.into(),
            },
            Err(_) => BITRATE_CFG_FILENAME.into(),
        };
        let data = read_to_string(libpath.clone())
            .map_err(|e| CanError::DeviceConfigError(format!("Unable to read `{}`: {:?}", libpath, e)))?;
        let result = serde_yaml::from_str(&data)
            .map_err(|e| CanError::DeviceConfigError(format!("Error parsing YAML: {:?}", e)))?;
        Ok(Self(Arc::new(result)))
    }

    pub fn new_can_chl_cfg(
        &self,
        dev_type: u32,
        can_type: u8,
        mode: u8,
        bitrate: u32,
        extra: CanChlCfgExt
    ) -> Result<CanChlCfg, CanError> {
        if self.0.contains_key(&dev_type.to_string()) {
            Ok(CanChlCfg::new(dev_type, can_type, mode, bitrate, extra, Arc::downgrade(&self.0)))
        }
        else {
            Err(CanError::DeviceConfigError(
                format!("device: {:?} is not configured in file!", dev_type)
            ))
        }
    }
}

fn get_fd_set(
    value: &CanChlCfg,
    cfg: &BitrateCfg,
    dbitrate: Option<u32>
) -> Result<(ZCanFdChlCfgSet, ZCanFdChlCfgSet), CanError> {
    let bitrate = value.bitrate;
    let bitrate_ctx = &cfg.bitrate;
    let dbitrate_ctx = &cfg.data_bitrate;
    let aset = bitrate_ctx
        .get(&bitrate.to_string())
        .ok_or(CanError::DeviceConfigError(format!("bitrate `{}` is not configured in file!", bitrate)))?;
    let dset=
        match dbitrate {
            Some(v) => {    // dbitrate is not None
                match dbitrate_ctx {
                    Some(ctx) => {  // dbitrate context is not None
                        match ctx.get(&v.to_string()) {
                            Some(value) => Ok(value),
                            None => Err(CanError::DeviceConfigError(format!("data bitrate `{}` is not configured in file!", v))),
                        }
                    },
                    None => {   // dbitrate context is None
                        match bitrate_ctx.get(&v.to_string()) {
                            Some(value) => Ok(value),
                            None => Err(CanError::DeviceConfigError(format!("data bitrate `{}` is not configured in file!", v))),
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

