use std::ffi::{c_uchar, c_uint};
use rs_can::CanError;
use super::constant::{ZLinCheckSumMode, ZLinMode};

#[allow(non_snake_case)]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ZLinChlCfg {
    linMode: c_uchar,   // 是否作为主机，0-从机，1-主机
    chkSumMode: c_uchar,// 校验方式，1-经典校验 2-增强校验 3-自动(对应eZLINChkSumMode的模式)
    maxLength: c_uchar, // 最大数据长度，8~64
    reserved: c_uchar,  // 保留
    linBaud: c_uint,    // 波特率，取值1000~20000
}

impl ZLinChlCfg {
    /// Create LIN channel configuration.
    /// max_len is required only windows.
    pub fn new(
        mode: u8,
        cs_mode: u8,
        bitrate: u32,
        max_len: Option<u8>
    ) -> Result<Self, CanError> {
        let mode = ZLinMode::try_from(mode)?;
        let cs_mode = ZLinCheckSumMode::try_from(cs_mode)?;
        match max_len {
            Some(v) => {
                match v {
                    8..=64 => {
                        match bitrate {
                            1000..=20000 => {
                                Ok(Self {
                                    linMode: mode as c_uchar,
                                    chkSumMode: cs_mode as c_uchar,
                                    maxLength: v,
                                    reserved: Default::default(),
                                    linBaud: bitrate
                                })
                            },
                            _ => Err(CanError::OtherError("parameter not supported".to_owned())),
                        }
                    },
                    _ => Err(CanError::OtherError("parameter not supported".to_owned())),
                }
            },
            None => {
                Ok(Self {
                    linMode: mode as c_uchar,
                    chkSumMode: cs_mode as c_uchar,
                    maxLength: Default::default(),
                    reserved: Default::default(),
                    linBaud: bitrate
                })
            },
        }
    }
}


