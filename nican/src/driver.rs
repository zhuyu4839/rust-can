use std::collections::HashMap;
use std::ffi::{c_char, c_void, CStr, CString};
use winapi::{shared::minwindef::HMODULE, um::{errhandlingapi::GetLastError, libloaderapi::{LoadLibraryA, GetProcAddress}, winnt::LPCSTR}};
use rs_can::{CanDriver, CanError, CanFilter, Frame};
use crate::{api::*, CanMessage, constant};

#[derive(Debug, Clone)]
struct NiCanContext {
    handle: NCTYPE_OBJH,
    filters: Vec<CanFilter>,
    bitrate: u32,
    log_errors: bool,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone)]
pub struct NiCan {
    _dll: HMODULE,
    channels: HashMap<String, NiCanContext>,
    ncConfig: unsafe extern "system" fn(NCTYPE_STRING, NCTYPE_UINT32, NCTYPE_ATTRID_P, NCTYPE_UINT32_P) -> NCTYPE_STATUS,
    ncOpenObject: unsafe extern "system" fn(NCTYPE_STRING, NCTYPE_OBJH_P) -> NCTYPE_STATUS,
    ncAction: unsafe extern "system" fn(NCTYPE_OBJH, NCTYPE_OPCODE, NCTYPE_UINT32) -> NCTYPE_STATUS,
    ncCloseObject: unsafe extern "system" fn(NCTYPE_OBJH) -> NCTYPE_STATUS,
    ncWrite: unsafe extern "system" fn(NCTYPE_OBJH, NCTYPE_UINT32, NCTYPE_ANY_P) -> NCTYPE_STATUS,
    ncRead: unsafe extern "system" fn(NCTYPE_OBJH, NCTYPE_UINT32, NCTYPE_ANY_P) -> NCTYPE_STATUS,
    ncWaitForState: unsafe extern "system" fn(NCTYPE_OBJH, NCTYPE_STATE, NCTYPE_DURATION, NCTYPE_STATE_P) -> NCTYPE_STATUS,
    ncStatusToString: unsafe extern "system" fn(NCTYPE_STATUS, NCTYPE_UINT32, NCTYPE_STRING) -> NCTYPE_STATUS,
}

unsafe impl Send for NiCan {}

impl NiCan {
    pub fn new(dll_path: Option<&str>) -> Result<Self, CanError> {
        let dll_path = dll_path.unwrap_or(r"Nican.dll");
        unsafe {
            let dll_cstr = CString::new(dll_path)
                .map_err(|e| CanError::OtherError(e.to_string()))?;
            let dll = LoadLibraryA(dll_cstr.as_ptr() as LPCSTR);
            if dll.is_null() {
                let code = GetLastError();
                return Err(CanError::DeviceConfigError(format!("Can't load library: {} code: {}", dll_path, code)));
            }

            Ok(Self {
                _dll: dll,
                channels: Default::default(),
                ncConfig: std::mem::transmute(GetProcAddress(dll, b"ncConfig\0".as_ptr() as LPCSTR)),
                ncOpenObject: std::mem::transmute(GetProcAddress(dll, b"ncOpenObject\0".as_ptr() as LPCSTR)),
                ncAction: std::mem::transmute(GetProcAddress(dll, b"ncAction\0".as_ptr() as LPCSTR)),
                ncCloseObject: std::mem::transmute(GetProcAddress(dll, b"ncCloseObject\0".as_ptr() as LPCSTR)),
                ncWrite: std::mem::transmute(GetProcAddress(dll, b"ncWrite\0".as_ptr() as LPCSTR)),
                ncRead: std::mem::transmute(GetProcAddress(dll, b"ncRead\0".as_ptr() as LPCSTR)),
                ncWaitForState: std::mem::transmute(GetProcAddress(dll, b"ncWaitForState\0".as_ptr() as LPCSTR)),
                ncStatusToString: std::mem::transmute(GetProcAddress(dll, b"ncStatusToString\0".as_ptr() as LPCSTR)),
            })
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

        let chl_ascii = CString::new(channel)
            .map_err(|e| CanError::OtherError(e.to_string()))?;
        let ret = unsafe {
            (self.ncConfig)(
                chl_ascii.clone().into_raw(),
                attr_id.len() as u32,
                attr_id.as_mut_ptr(),
                attr_val.as_mut_ptr(),
            )
        };
        if ret != 0 {
            return Err(CanError::DeviceOpenFailed);
        }

        let mut handle = 0;
        let ret = unsafe { (self.ncOpenObject)(chl_ascii.into_raw(), &mut handle) };
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
                let ret = unsafe { (self.ncAction)(ctx.handle, NC_OP_RESET, 0) };

                self.check_status(channel.as_str(), ret)
                    .map_err(|r| {
                        log::warn!(
                            "{} error {} when reset",
                            Self::channel_info(&channel),
                            self.status_to_str(r)
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
                let ret = unsafe { (self.ncCloseObject)(ctx.handle) };
                self.channels.remove(&channel);

                self.check_status(channel.as_str(), ret)
                    .map_err(|r| {
                        log::warn!(
                            "{} error {} when close",
                            Self::channel_info(&channel),
                            self.status_to_str(r)
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
                    (self.ncWrite)(
                        ctx.handle,
                        std::mem::size_of::<NCTYPE_CAN_FRAME>() as u32,
                        &raw_msg as *const NCTYPE_CAN_FRAME as *mut c_void,
                    )
                };

                if let Err(r) = self.check_status(channel.as_str(), ret) {
                    log::warn!(
                        "{} error {} when transmit",
                        Self::channel_info(&channel),
                        self.status_to_str(r)
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
                if let Err(ret) = self.wait_for_state(channel.as_str(), ctx.handle, timeout) {
                    if ret == constant::CanErrFunctionTimeout {
                        log::warn!("{} wait for state timeout", Self::channel_info(&channel));
                    }
                    return Err(CanError::TimeoutError(Self::channel_info(&channel)));
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
                    (self.ncRead)(
                        ctx.handle,
                        std::mem::size_of::<NCTYPE_CAN_STRUCT>() as u32,
                        &raw_msg as *const NCTYPE_CAN_STRUCT as *mut c_void,
                    )
                };

                if let Err(r) = self.check_status(channel.as_str(), ret) {
                    log::warn!(
                        "{} error {} when receive",
                        Self::channel_info(&channel),
                        self.status_to_str(r)
                    );
                    return Err(CanError::OperationError(Self::channel_info(&channel)));
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

    fn wait_for_state(&self, channel: &str, handle: NCTYPE_OBJH, timeout: Option<u32>) -> Result<(), i32> {
        let timeout = timeout.unwrap_or(NC_DURATION_INFINITE);

        let mut state = 0;
        let ret = unsafe { (self.ncWaitForState)(handle, NC_ST_READ_AVAIL, timeout, &mut state) };

        self.check_status(channel, ret)
    }

    fn check_status(&self, channel: &str, result: i32) -> Result<(), i32> {
        if result > 0 {
            log::warn!("{} {}", Self::channel_info(channel), self.status_to_str(result));
            Ok(())
        } else if result < 0 {
            Err(result)
        } else {
            Ok(())
        }
    }

    fn status_to_str(&self, code: i32) -> String {
        let mut err = [0u8; 1024];
        unsafe { (self.ncStatusToString)(code, err.len() as u32, err.as_mut_ptr() as *mut c_char) };
        let cstr = unsafe { CStr::from_ptr(err.as_ptr() as *const c_char) };

        cstr.to_str().unwrap_or("Unknown").to_string()
    }
}

impl CanDriver for NiCan {
    type Channel = String;
    type Frame = CanMessage;

    #[inline]
    fn is_closed(&self) -> bool {
        self.channels.is_empty()
    }

    #[inline]
    fn opened_channels(&self) -> Vec<Self::Channel> {
        self.channels.keys()
            .map(|v| v.clone())
            .collect()
    }

    #[inline]
    fn transmit(&self, msg: Self::Frame, _: Option<u32>) -> Result<(), CanError> {
        self.transmit_can(msg)
    }

    #[inline]
    fn receive(&self, channel: Self::Channel, timeout: Option<u32>) -> Result<Vec<Self::Frame>, CanError> {
        self.receive_can(channel, timeout)
    }

    #[inline]
    fn shutdown(&mut self) {
        self.channels.iter()
            .for_each(|(c, ctx)| {
                let ret = unsafe { (self.ncCloseObject)(ctx.handle) };

                if let Err(e) = self.check_status(c, ret) {
                    log::warn!(
                        "{} error {} when close",
                        Self::channel_info(c),
                        self.status_to_str(e)
                    );
                }
            });

        self.channels.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::NiCan;
    use crate::CanMessage;
    use std::time::Duration;
    use rs_can::{CanDriver, Frame, Id};
    use rs_can::isotp::IsoTpAdapter;

    #[ignore]   // device required
    #[test]
    fn api() -> anyhow::Result<()> {
        let channel = "CAN0";
        let mut driver = NiCan::new(None)?;
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
        let mut driver = NiCan::new(None)?;
        driver.open(channel, vec![], 500_000, true)?;

        let mut adapter = IsoTpAdapter::new(driver.clone());
        adapter.start(100);

        let data = vec![0x02, 0x10, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00];
        let mut count = 0;
        loop {
            let mut msg = CanMessage::new(Id::from(0x7DF), data.as_slice()).unwrap();
            msg.set_channel(channel.into());
            adapter.sender().send(msg)?;

            std::thread::sleep(Duration::from_millis(100));

            count += 1;
            if count > 10 {
                break;
            }
        }

        adapter.stop();
        driver.shutdown();

        Ok(())
    }
}
