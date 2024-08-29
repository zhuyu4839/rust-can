// #[cfg(not(all(target_os = "windows", target_arch = "x86")))]
// compile_error!("This crate can only be compiled for 32-bit Windows.");

use crate::frame::CanMessage;
use rs_can::error::CanError;
use rs_can::{CanFilter, Frame};
use std::ffi::{c_char, c_void, CStr, CString};

mod api;
mod constant;
mod frame;
use api::*;

#[derive(Debug, Clone)]
pub struct NiCan {
    handle: NCTYPE_OBJH,
    channel: String,
    filters: Vec<CanFilter>,
    bitrate: u32,
    log_errors: bool,
}

impl NiCan {
    pub fn new(
        channel: &str,
        filters: Vec<CanFilter>,
        bitrate: u32,
        log_errors: bool,
    ) -> Result<Self, CanError> {
        let mut attr_id = vec![NC_ATTR_START_ON_OPEN, NC_ATTR_LOG_COMM_ERRS];
        let mut attr_val = vec![1, if log_errors { 1 } else { 0 }];

        match filters.len() {
            0 => {
                attr_id.extend([
                    NC_ATTR_CAN_COMP_STD,
                    NC_ATTR_CAN_MASK_STD,
                    NC_ATTR_CAN_COMP_XTD,
                    NC_ATTR_CAN_MASK_XTD,
                ]);
                attr_val.extend([0; 4])
            }
            _ => filters.iter().for_each(|f| {
                attr_id.extend([NC_ATTR_CAN_COMP_XTD, NC_ATTR_CAN_MASK_XTD]);
                if f.extended {
                    attr_val.extend([f.can_id | NC_FL_CAN_ARBID_XTD, f.can_mask]);
                } else {
                    attr_val.extend([f.can_id, f.can_mask]);
                }
            }),
        }

        attr_id.push(NC_ATTR_BAUD_RATE);
        attr_val.push(bitrate);

        let chl_ascii = CString::new(channel).expect("can't convert str to `CString`");
        let ret = unsafe {
            ncConfig(
                chl_ascii.clone().into_raw(),
                attr_id.len() as u32,
                attr_id.as_mut_ptr(),
                attr_val.as_mut_ptr(),
            )
        };
        if ret != 0 {
            return Err(CanError::InitializationError);
        }

        let mut handle = 0;
        let ret = unsafe { ncOpenObject(chl_ascii.into_raw(), &mut handle) };
        if ret != 0 {
            return Err(CanError::InitializationError);
        }

        Ok(Self {
            handle,
            channel: channel.into(),
            filters,
            bitrate,
            log_errors,
        })
    }

    pub fn transmit(&self, msg: CanMessage) {
        let raw_msg = msg.into();

        let ret = unsafe {
            ncWrite(
                self.handle,
                std::mem::size_of::<NCTYPE_CAN_FRAME>() as u32,
                &raw_msg as *const NCTYPE_CAN_FRAME as *mut c_void,
            )
        };

        if let Err(r) = self.check_status(ret) {
            log::warn!(
                "{} error {} when transmit",
                self.channel_info(),
                Self::status_to_str(r)
            )
        }
    }

    pub fn receive(&self, timeout: Option<u32>) -> Option<CanMessage> {
        if let Err(ret) = self.wait_for_state(timeout) {
            if ret == constant::CanErrFunctionTimeout {
                log::debug!("{} wait for state timeout", self.channel_info());
            }
            return None;
        }

        let raw_msg = NCTYPE_CAN_STRUCT {
            Timestamp: NCTYPE_UINT64 {
                LowPart: Default::default(),
                HighPart: Default::default(),
            },
            ArbitrationId: Default::default(),
            FrameType: Default::default(),
            DataLength: Default::default(),
            Data: Default::default(),
        };

        let ret = unsafe {
            ncRead(
                self.handle,
                std::mem::size_of::<NCTYPE_CAN_STRUCT>() as u32,
                &raw_msg as *const NCTYPE_CAN_STRUCT as *mut c_void,
            )
        };

        if let Err(r) = self.check_status(ret) {
            log::warn!(
                "{} error {} when receive",
                self.channel_info(),
                Self::status_to_str(r)
            );
            return None;
        }

        let mut msg = <NCTYPE_CAN_STRUCT as Into<Option<CanMessage>>>::into(raw_msg)?;
        msg.set_channel(self.channel.clone());

        Some(msg)
    }

    #[inline]
    pub fn reset(&self) {
        let ret = unsafe { ncAction(self.handle, NC_OP_RESET, 0) };
        if let Err(r) = self.check_status(ret) {
            log::warn!(
                "{} error {} when reset",
                self.channel_info(),
                Self::status_to_str(r)
            );
        }
    }

    #[inline]
    pub fn close(&self) {
        let ret = unsafe { ncCloseObject(self.handle) };
        if let Err(r) = self.check_status(ret) {
            log::warn!(
                "{} error {} when close",
                self.channel_info(),
                Self::status_to_str(r)
            );
        }
    }

    #[inline]
    pub fn channel_info(&self) -> String {
        format!("NI-CAN: {}", self.channel)
    }

    #[inline]
    pub fn filters(&self) -> &Vec<CanFilter> {
        &self.filters
    }

    #[inline]
    pub fn bitrate(&self) -> u32 {
        self.bitrate
    }

    #[inline]
    pub fn is_log_errors(&self) -> bool {
        self.log_errors
    }

    fn wait_for_state(&self, timeout: Option<u32>) -> Result<(), i32> {
        let timeout = timeout.unwrap_or(NC_DURATION_INFINITE);

        let mut state = 0;
        let ret = unsafe { ncWaitForState(self.handle, NC_ST_READ_AVAIL, timeout, &mut state) };

        self.check_status(ret)
    }

    fn check_status(&self, result: i32) -> Result<(), i32> {
        if result > 0 {
            log::warn!("{} {}", self.channel_info(), Self::status_to_str(result));
            Ok(())
        } else if result < 0 {
            Err(result)
        } else {
            Ok(())
        }
    }

    fn status_to_str(code: i32) -> String {
        let mut err = [0u8; 1024];
        unsafe { ncStatusToString(code, err.len() as u32, err.as_mut_ptr() as *mut c_char) };
        let cstr = unsafe { CStr::from_ptr(err.as_ptr() as *const c_char) };

        cstr.to_str().unwrap_or("Unknown").to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::NiCan;
    use crate::frame::CanMessage;
    use rs_can::{Frame, Id};
    use std::time::Duration;

    #[test]
    fn api() -> anyhow::Result<()> {
        let driver = NiCan::new("CAN0".into(), vec![], 500_000, true)?;

        let data = vec![0x02, 0x10, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00];
        let mut count = 0;
        loop {
            driver.transmit(CanMessage::new(Id::from(0x7DF), data.as_slice()).unwrap());

            std::thread::sleep(Duration::from_millis(5));
            println!("{:?}", driver.receive(Some(10)));
            std::thread::sleep(Duration::from_millis(100));

            count += 1;
            if count > 10 {
                break;
            }
        }

        driver.close();

        Ok(())
    }
}
