use std::collections::HashMap;
use std::ffi::{c_char, c_void, CStr, CString};
use isotp_rs::can::frame::Frame;
use isotp_rs::device::Driver;
use rs_can::CanFilter;
use rs_can::error::CanError;
use crate::api::*;
use crate::constant;
use crate::frame::CanMessage;

#[derive(Debug, Clone)]
struct NiCanContext {
    handle: NCTYPE_OBJH,
    filters: Vec<CanFilter>,
    bitrate: u32,
    log_errors: bool,
}

#[derive(Debug, Clone)]
pub struct NiCan {
    channels: HashMap<String, NiCanContext>
}

impl NiCan {
    pub fn new() -> Self {
        Self {
            channels: Default::default(),
        }
    }

    pub fn open(&mut self,
                channel: &str,
                filters: Vec<CanFilter>,
                bitrate: u32,
                log_errors: bool,
    ) -> Result<(), CanError> {
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
            return Err(CanError::DeviceConfigFailed);
        }

        let mut handle = 0;
        let ret = unsafe { ncOpenObject(chl_ascii.into_raw(), &mut handle) };
        if ret != 0 {
            return Err(CanError::DeviceOpenFailed);
        }

        self.channels.insert(channel.into(), NiCanContext {
            handle,
            filters,
            bitrate,
            log_errors,
        });

        Ok(())
    }

    pub fn reset(&mut self, channel: String) -> Result<(), CanError> {
        match self.channels.get(&channel) {
            Some(ctx) => {
                let ret = unsafe { ncAction(ctx.handle, NC_OP_RESET, 0) };

                Self::check_status(channel.as_str(), ret)
                    .map_err(|r| {
                        log::warn!(
                            "{} error {} when reset",
                            Self::channel_info(channel.as_str()),
                            Self::status_to_str(r)
                        );

                        CanError::OperationError(channel)
                    })
            },
            None => Err(CanError::ChannelNotOpened(channel)),
        }
    }

    pub fn close(&mut self, channel: String) -> Result<(), CanError> {
        match self.channels.get(&channel) {
            Some(ctx) => {
                let ret = unsafe { ncCloseObject(ctx.handle) };
                self.channels.remove(&channel);

                Self::check_status(channel.as_str(), ret)
                    .map_err(|r| {
                        log::warn!(
                            "{} error {} when close",
                            Self::channel_info(channel.as_str()),
                            Self::status_to_str(r)
                        );

                        CanError::OperationError(channel)
                    })
            },
            None => Err(CanError::ChannelNotOpened(channel)),
        }
    }

    pub fn transmit_can(&self, msg: CanMessage) -> Result<(), CanError> {
        let channel = msg.channel();
        match self.channels.get(&channel) {
            Some(ctx) => {
                let raw_msg = msg.into();

                let ret = unsafe {
                    ncWrite(
                        ctx.handle,
                        std::mem::size_of::<NCTYPE_CAN_FRAME>() as u32,
                        &raw_msg as *const NCTYPE_CAN_FRAME as *mut c_void,
                    )
                };

                if let Err(r) = Self::check_status(channel.as_str(), ret) {
                    log::warn!(
                        "{} error {} when transmit",
                        Self::channel_info(channel.as_str()),
                        Self::status_to_str(r)
                    )
                }

                Ok(())
            },
            None => Err(CanError::ChannelNotOpened(channel)),
        }
    }

    pub fn receive_can(&self, channel: String, timeout: Option<u32>) -> Result<Vec<CanMessage>, CanError> {
        match self.channels.get(&channel) {
            Some(ctx) => {
                if let Err(ret) = Self::wait_for_state(channel.as_str(), ctx.handle, timeout) {
                    if ret == constant::CanErrFunctionTimeout {
                        log::debug!("{} wait for state timeout", Self::channel_info(channel.as_str()));
                    }
                    return Err(CanError::TimeoutError(Self::channel_info(channel.as_str())));
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
                        ctx.handle,
                        std::mem::size_of::<NCTYPE_CAN_STRUCT>() as u32,
                        &raw_msg as *const NCTYPE_CAN_STRUCT as *mut c_void,
                    )
                };

                if let Err(r) = Self::check_status(channel.as_str(), ret) {
                    log::warn!(
                        "{} error {} when receive",
                        Self::channel_info(channel.as_str()),
                        Self::status_to_str(r)
                    );
                    return Err(CanError::OperationError(Self::channel_info(channel.as_str())));
                }

                let mut msg = <NCTYPE_CAN_STRUCT as TryInto<CanMessage>>::try_into(raw_msg)?;
                msg.set_channel(channel.clone());

                Ok(vec![msg, ])
            },
            None => Err(CanError::ChannelNotOpened(channel)),
        }
    }

    #[inline]
    pub fn channel_info(channel: &str) -> String {
        format!("NI-CAN: {}", channel)
    }

    #[inline]
    pub fn filters(&self, channel: String) -> Result<Vec<CanFilter>, CanError> {
        self.channel_util(channel, |ctx| Ok(ctx.filters.clone()))
    }

    #[inline]
    pub fn bitrate(&self, channel: String) -> Result<u32, CanError> {
        self.channel_util(channel, |ctx| Ok(ctx.bitrate))
    }

    #[inline]
    pub fn is_log_errors(&self, channel: String) -> Result<bool, CanError> {
        self.channel_util(channel, |ctx| Ok(ctx.log_errors))
    }

    #[inline]
    fn channel_util<R>(&self,
                       channel: String,
                       cb: fn(ctx: &NiCanContext) -> Result<R, CanError>
    ) -> Result<R, CanError> {
        match self.channels.get(&channel) {
            Some(ctx) => cb(ctx),
            None => Err(CanError::ChannelNotOpened(channel)),
        }
    }

    fn wait_for_state(channel: &str, handle: NCTYPE_OBJH, timeout: Option<u32>) -> Result<(), i32> {
        let timeout = timeout.unwrap_or(NC_DURATION_INFINITE);

        let mut state = 0;
        let ret = unsafe { ncWaitForState(handle, NC_ST_READ_AVAIL, timeout, &mut state) };

        Self::check_status(channel, ret)
    }

    fn check_status(channel: &str, result: i32) -> Result<(), i32> {
        if result > 0 {
            log::warn!("{} {}", Self::channel_info(channel), Self::status_to_str(result));
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

impl Driver for NiCan {
    type Error = CanError;
    type C = String;
    type F = CanMessage;

    #[inline]
    fn opened_channels(&self) -> Vec<Self::C> {
        self.channels.keys()
            .map(|v| v.clone())
            .collect()
    }

    #[inline]
    fn is_closed(&self) -> bool {
        self.channels.is_empty()
    }

    #[inline]
    fn transmit(&self, msg: Self::F, _: Option<u32>) -> Result<(), Self::Error> {
        self.transmit_can(msg)
    }

    #[inline]
    fn receive(&self, channel: Self::C, timeout: Option<u32>) -> Result<Vec<Self::F>, Self::Error> {
        self.receive_can(channel, timeout)
    }

    #[inline]
    fn shutdown(&mut self) {
        self.channels.iter()
            .for_each(|(c, ctx)| {
                let ret = unsafe { ncCloseObject(ctx.handle) };

                if let Err(e) = Self::check_status(c, ret) {
                    log::warn!(
                        "{} error {} when close",
                        Self::channel_info(c),
                        Self::status_to_str(e)
                    );
                }
            });

        self.channels.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::NiCan;
    use crate::frame::CanMessage;
    use std::time::Duration;
    use isotp_rs::can::{frame::Frame, identifier::Id};
    use isotp_rs::can::driver::SyncCan;
    use isotp_rs::device::Driver;

    #[ignore]   // device required
    #[test]
    fn api() -> anyhow::Result<()> {
        let channel = "CAN0";
        let mut driver = NiCan::new();
        driver.open(channel, vec![], 500_000, true)?;

        let data = vec![0x02, 0x10, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00];
        let mut count = 0;
        loop {
            let mut msg = CanMessage::new(Id::from(0x7DF), data.as_slice()).unwrap();
            msg.set_channel(channel.into());
            driver.transmit(msg, None)?;

            std::thread::sleep(Duration::from_millis(5));
            if let Ok(recv) = driver.receive(channel.into(), Some(10)) {
                recv.into_iter()
                    .for_each(|msg| println!("{}", msg));
            }
            std::thread::sleep(Duration::from_millis(100));

            count += 1;
            if count > 10 {
                break;
            }
        }

        driver.close(channel.into())?;

        Ok(())
    }

    #[ignore]   // device required
    #[test]
    fn isotp() -> anyhow::Result<()> {
        let channel = "CAN0";
        let mut driver = NiCan::new();
        driver.open(channel, vec![], 500_000, true)?;

        let mut uni = SyncCan::new(driver.clone());
        uni.sync_start(100);

        let data = vec![0x02, 0x10, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00];
        let mut count = 0;
        loop {
            let mut msg = CanMessage::new(Id::from(0x7DF), data.as_slice()).unwrap();
            msg.set_channel(channel.into());
            uni.sender().send(msg)?;

            std::thread::sleep(Duration::from_millis(100));

            count += 1;
            if count > 10 {
                break;
            }
        }

        uni.stop();
        driver.shutdown();

        Ok(())
    }
}
