use std::{collections::HashMap, fs::read_to_string, ffi::{c_uchar, c_uint, c_ushort}};
use serde::Deserialize;
use rs_can::{CanError, ChannelConfig};
use crate::can::{ZCanFilterType, constant::{BITRATE_CFG_FILENAME, TIMING0, TIMING1, ZCAN_ENV, ZCAN_VAR}};
use crate::{ACC_CODE, ACC_MASK, CHANNEL_MODE, FILTER};

#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Debug, Default, Copy, Clone)]
pub enum ZCanChlType {
    #[default]
    CAN = 0,
    CANFD_ISO = 1,
    CANFD_NON_ISO = 2,
}

impl TryFrom<u8> for ZCanChlType {
    type Error = CanError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ZCanChlType::CAN),
            1 => Ok(ZCanChlType::CANFD_ISO),
            2 => Ok(ZCanChlType::CANFD_NON_ISO),
            _ => Err(CanError::other_error("parameter not supported")),
        }
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub enum ZCanChlMode {
    #[default]
    Normal = 0,
    ListenOnly = 1,
}

impl TryFrom<u8> for ZCanChlMode {
    type Error = CanError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ZCanChlMode::Normal),
            1 => Ok(ZCanChlMode::ListenOnly),
            _ => Err(CanError::other_error("parameter not supported")),
        }
    }
}

/// The deserialize object mapped to configuration file context.
#[derive(Debug, Deserialize)]
pub(crate) struct BitrateCfg {
    pub(crate) bitrate: HashMap<String, HashMap<String, u32>>,
    pub(crate) clock: Option<u32>,
    #[allow(unused)]
    pub(crate) data_bitrate: Option<HashMap<String, HashMap<String, u32>>>
}

#[derive(Debug)]
pub(crate) struct CanChlCfgContext(pub(crate) HashMap<String, BitrateCfg>);

impl CanChlCfgContext {
    pub fn new() -> Result<Self, CanError> {
        let libpath = match dotenvy::from_filename(ZCAN_ENV) {
            Ok(_) => match std::env::var(ZCAN_VAR){
                Ok(v) => format!("{}/{}", v, BITRATE_CFG_FILENAME),
                Err(_) => BITRATE_CFG_FILENAME.into(),
            },
            Err(_) => BITRATE_CFG_FILENAME.into(),
        };
        let data = read_to_string(&libpath)
            .map_err(|e| CanError::OtherError(format!("Unable to read `{}`: {:?}", libpath, e)))?;
        let result = serde_yaml::from_str(&data)
            .map_err(|e| CanError::OtherError(format!("Error parsing YAML: {:?}", e)))?;

        Ok(Self(result))
    }
}

/// Linux USBCAN USBCAN_4E(8_E) USBCANFD_800U and windows
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct ZCanChlCfgInner {
    pub(crate) acc_code: c_uint,
    pub(crate) acc_mask: c_uint,
    #[allow(dead_code)]
    reserved: c_uint,
    pub(crate) filter: c_uchar,
    pub(crate) timing0: c_uchar,
    pub(crate) timing1: c_uchar,
    pub(crate) mode: c_uchar,
}

impl ZCanChlCfgInner {
    #[inline(always)]
    pub(crate) fn new(
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

    pub(crate) fn try_from_with(
        bc: &BitrateCfg,
        cfg: &ChannelConfig
    ) -> Result<Self, CanError> {
        let bitrate = cfg.bitrate();
        match bc.bitrate.get(&bitrate.to_string()) {
            Some(v) => {
                let &timing0 = v.get(TIMING0)
                    .ok_or(CanError::OtherError(format!("`{}` is not configured in file!", TIMING0)))?;
                let &timing1 = v.get(TIMING1)
                    .ok_or(CanError::OtherError(format!("`{}` is not configured in file!", TIMING1)))?;

                Self::new(
                    cfg.get_other::<u8>(CHANNEL_MODE)?
                        .unwrap_or(ZCanChlMode::Normal as u8),
                    timing0,
                    timing1,
                    cfg.get_other::<u8>(FILTER)?
                        .unwrap_or(0xFF),
                    cfg.get_other::<u32>(ACC_CODE)?,
                    cfg.get_other::<u32>(ACC_MASK)?,
                )
            },
            None => Err(CanError::OtherError(
                format!("the bitrate: `{}` is not configured", bitrate)
            )),
        }
    }
}

/// Linux USBCAN_4E_8E USBCANFD_800U and windows
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct ZCanFdChlCfgInner {
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
impl ZCanFdChlCfgInner {
    #[inline(always)]
    pub(crate) fn new(
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

