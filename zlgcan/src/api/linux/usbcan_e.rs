use std::ffi::{c_uchar, c_uint, CString};
use dlopen2::symbor::{Symbol, SymBorApi};
use rs_can::CanError;

use crate::can::{CanChlCfg, ZCanChlCfgV1, ZCanChlError, ZCanChlErrorV2, ZCanChlStatus, ZCanFrameType, ZCanFrameV3};
use crate::device::{Handler, IProperty, SetValueFunc, ZCanDeviceType, ZChannelContext, ZDeviceContext, ZDeviceInfo};
use crate::constant::{channel_bitrate, channel_work_mode};
use crate::api::{ZCanApi, ZCloudApi, ZDeviceApi, ZLinApi};

#[allow(non_snake_case)]
#[derive(Debug, Clone, SymBorApi)]
pub(crate) struct USBCANEApi<'a> {
    /// DEVICE_HANDLE ZCAN_OpenDevice(UINT device_type, UINT device_index, UINT reserved);
    ZCAN_OpenDevice: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_index: c_uint, reserved: c_uint) -> c_uint>,
    /// INT ZCAN_CloseDevice(DEVICE_HANDLE device_handle);
    ZCAN_CloseDevice: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint) -> c_uint>,
    /// INT ZCAN_GetDeviceInf(DEVICE_HANDLE device_handle, ZCAN_DEVICE_INFO* pInfo);
    ZCAN_GetDeviceInf: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint, info: *mut ZDeviceInfo) -> c_uint>,
    /// CHANNEL_HANDLE ZCAN_InitCAN(DEVICE_HANDLE device_handle, UINT can_index, ZCAN_CHANNEL_INIT_CONFIG* pInitConfig);
    ZCAN_InitCAN: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint, channel: c_uint, cfg: *const ZCanChlCfgV1) -> c_uint>,
    /// INT ZCAN_StartCAN(CHANNEL_HANDLE channel_handle);
    ZCAN_StartCAN: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint) -> c_uint>,
    /// INT ZCAN_ResetCAN(CHANNEL_HANDLE channel_handle);
    ZCAN_ResetCAN: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint) -> c_uint>,
    /// INT ZCAN_ClearBuffer(CHANNEL_HANDLE channel_handle);
    ZCAN_ClearBuffer: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint) -> c_uint>,
    /// INT ZCAN_ReadChannelErrInfo(CHANNEL_HANDLE channel_handle, ZCAN_CHANNEL_ERR_INFO* pErrInfo);
    ZCAN_ReadChannelErrInfo: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, err: *mut ZCanChlError) -> c_uint>,
    /// INT ZCAN_ReadChannelStatus(CHANNEL_HANDLE channel_handle, ZCAN_CHANNEL_STATUS* pCANStatus);
    ZCAN_ReadChannelStatus: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, status: *mut ZCanChlStatus) -> c_uint>,
    /// INT ZCAN_Transmit(CHANNEL_HANDLE channel_handle, ZCAN_Transmit_Data* pTransmit, UINT len);
    ZCAN_Transmit: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, frames: *const ZCanFrameV3, len: c_uint) -> c_uint>,
    /// INT ZCAN_GetReceiveNum(CHANNEL_HANDLE channel_handle, BYTE type);
    ZCAN_GetReceiveNum: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, msg: c_uchar) -> c_uint>,
    /// INT ZCAN_Receive(CHANNEL_HANDLE channel_handle, ZCAN_Receive_Data* pReceive, UINT len, INT wait_time);
    ZCAN_Receive: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, frames: *const ZCanFrameV3, size: c_uint, timeout: c_uint) -> c_uint>,
    /// INT ZCAN_TransmitFD(CHANNEL_HANDLE channel_handle, ZCAN_TransmitFD_Data* pTransmit, UINT len);
    //ZCAN_TransmitFD: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, frames: *const ZCanFdFrame, len: c_uint) -> c_uint>,
    /// INT ZCAN_ReceiveFD(CHANNEL_HANDLE channel_handle, ZCAN_ReceiveFD_Data* pReceive, UINT len, INT wait_time);
    //ZCAN_ReceiveFD: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, frames: *const ZCanFdFrame, size: c_uint, timeout: c_uint) -> c_uint>,

    /// IProperty* GetIProperty(DEVICE_HANDLE device_handle);   //获取属性接口
    GetIProperty: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint) -> *const IProperty>,
    /// INT ReleaseIProperty(IProperty * pIProperty);
    ReleaseIProperty: Symbol<'a, unsafe extern "C" fn(p: *const IProperty) -> c_uint>,
}

impl USBCANEApi<'_> {
    pub(crate) const INVALID_DEVICE_HANDLE: u32 = 0;
    pub(crate) const INVALID_CHANNEL_HANDLE: u32 = 0;
    pub(crate) const STATUS_OK: u32 = 0;
    pub(crate) fn init_can_chl_ex(
        &self,
        dev_hdl: &mut Handler,
        channels: u8,
        cfg: &Vec<CanChlCfg>,
    ) -> Result<(), CanError> {
        let p = self.self_get_property(dev_hdl.device_context())?;
        let set_value_func = p.SetValue;
        let mut error = None;
        for (idx, cfg) in cfg.iter().enumerate() {
            let idx = idx as u8;
            if idx >= channels {
                log::warn!("ZLGCAN - the length of CAN channel configuration is out of channels!");
                break;
            }

            if let Some(chl_hdl) = dev_hdl.find_can(idx) {
                self.reset_can_chl(chl_hdl).unwrap_or_else(|e| log::warn!("{}", e));
                dev_hdl.remove_can(idx);
            }

            match self.start_channel(dev_hdl, idx, set_value_func, cfg) {
                Ok(context) => {
                    dev_hdl.add_can(idx, context);
                },
                Err(e) => {
                    error = Some(e);
                    break;
                }
            }
        }
        self.release_property(&p)?;

        match error {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }
    #[inline]
    fn start_channel(
        &self,
        dev_hdl: &mut Handler,
        channel: u8,
        set_value_func: SetValueFunc,
        cfg: &CanChlCfg
    ) -> Result<ZChannelContext, CanError> {
        let mut context = ZChannelContext::new(dev_hdl.device_context().clone(), channel, None);
        self.init_can_chl(&mut context, cfg)?; // ZCAN_InitCAN]
        // self.usbcan_4e_api.reset_can_chl(chl_hdl).unwrap_or_else(|e| log::warn!("{}", e));
        let (chl_hdl, channel) = (context.channel_handler()?, context.channel());
        self.set_channel(channel, set_value_func, cfg)?;

        match unsafe { (self.ZCAN_StartCAN)(chl_hdl) as u32 } {
            Self::STATUS_OK => Ok(context),
            code => Err(CanError::OtherError(format!("`ZCAN_StartCAN` ret: {}", code))),
        }
    }

    fn set_channel(
        &self,
        channel: u8,
        func: SetValueFunc,
        cfg: &CanChlCfg
    ) -> Result<(), CanError> {
        unsafe {
            let func = func.ok_or(CanError::OtherError("method not supported".to_owned()))?;
            let cmd_path = CString::new(channel_bitrate(channel))
                .map_err(|e| CanError::OtherError(e.to_string()))?;
            let bitrate = CString::new(cfg.bitrate().to_string())
                .map_err(|e| CanError::OtherError(e.to_string()))?;
            match func(cmd_path.as_ptr(), bitrate.as_ptr()) as u32 {
                Self::STATUS_OK => Ok(()),
                code => Err(CanError::OtherError(format!("{:?}, SetValue failed ret: {}", cmd_path, code))),
            }?;

            let cmd_path = CString::new(channel_work_mode(channel))
                .map_err(|e| CanError::OtherError(e.to_string()))?;
            let mode = CString::new(cfg.mode().to_string())
                .map_err(|e| CanError::OtherError(e.to_string()))?;
            match func(cmd_path.as_ptr(), mode.as_ptr()) as u32 {
                Self::STATUS_OK => Ok(()),
                code => Err(CanError::OtherError(format!("{:?}, SetValue failed ret: {}", cmd_path, code))),
            }
        }
    }

    fn self_get_property(&self, context: &ZDeviceContext) -> Result<IProperty, CanError> {
        let ret = unsafe { (self.GetIProperty)(context.device_handler()?) };
        if ret.is_null() {
            Err(CanError::OtherError(format!("`GetIProperty` ret: {}", 0)))
        }
        else {
            unsafe { Ok(*ret) }
        }
    }
}

impl ZDeviceApi for USBCANEApi<'_> {
    fn open(&self, context: &mut ZDeviceContext) -> Result<(), CanError> {
        let (dev_type, dev_idx) = (context.device_type(), context.device_index());
        match unsafe { (self.ZCAN_OpenDevice)(dev_type as u32, dev_idx, 0) } as u32 {
            Self::INVALID_DEVICE_HANDLE => Err(CanError::OtherError(format!("`ZCAN_OpenDevice` ret: {}", Self::INVALID_DEVICE_HANDLE))),
            handler => {
                context.set_device_handler(handler);
                Ok(())
            },
        }
    }

    fn close(&self, context: &ZDeviceContext) -> Result<(), CanError> {
        match unsafe { (self.ZCAN_CloseDevice)(context.device_handler()?) } as u32 {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::OtherError(format!("ZCAN_CloseDevice ret: {}", code))),
        }
    }

    fn read_device_info(&self, context: &ZDeviceContext) -> Result<ZDeviceInfo, CanError> {
        let mut info = ZDeviceInfo::default();
        match unsafe { (self.ZCAN_GetDeviceInf)(context.device_handler()?, &mut info) } as u32 {
            Self::STATUS_OK => Ok(info),
            code => Err(CanError::OtherError(format!("ZCAN_GetDeviceInf ret: {}", code))),
        }
    }

    fn get_property(&self, context: &ZChannelContext) -> Result<IProperty, CanError> {
        self.self_get_property(context.device_context())
    }

    fn release_property(&self, p: &IProperty) -> Result<(), CanError> {
        match unsafe { (self.ReleaseIProperty)(p) } {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::OtherError(format!("`ReleaseIProperty` ret: {}", code))),
        }
    }
}

impl ZCanApi for USBCANEApi<'_> {
    type Frame = ZCanFrameV3;
    type FdFrame = ();
    fn init_can_chl(&self, context: &mut ZChannelContext, cfg: &CanChlCfg) -> Result<(), CanError> {
        let dev_hdl = context.device_handler()?;
        let channel = context.channel() as u32;
        unsafe {
            let dev_type = cfg.device_type()?;
            let handler = match dev_type {
                ZCanDeviceType::ZCAN_USBCAN_4E_U => {
                    match (self.ZCAN_InitCAN)(dev_hdl, channel, std::ptr::null()) as u32 {
                        Self::INVALID_CHANNEL_HANDLE =>
                            Err(CanError::OtherError(format!("`ZCAN_InitCAN` ret: {}", Self::INVALID_CHANNEL_HANDLE))),
                        handler => Ok(handler),
                    }
                },
                ZCanDeviceType::ZCAN_USBCAN_8E_U => {
                    let cfg = ZCanChlCfgV1::try_from(cfg)?;
                    match (self.ZCAN_InitCAN)(dev_hdl, channel, &cfg) as u32 {
                        Self::INVALID_CHANNEL_HANDLE => Err(CanError::OtherError(format!("ZCAN_InitCAN ret: {}", Self::INVALID_CHANNEL_HANDLE))),
                        handler => {
                            match (self.ZCAN_StartCAN)(handler) as u32 {
                                Self::STATUS_OK => Ok(handler),
                                code => Err(CanError::OtherError(format!("`ZCAN_StartCAN` ret: {}", code))),
                            }
                        }
                    }
                },
                _ => Err(CanError::DeviceNotSupported),
            }?;

            context.set_channel_handler(Some(handler));
            Ok(())
        }
    }

    fn reset_can_chl(&self, context: &ZChannelContext) -> Result<(), CanError> {
        match unsafe { (self.ZCAN_ResetCAN)(context.channel_handler()?) } as u32 {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::OtherError(format!("`ZCAN_ResetCAN` ret: {}", code))),
        }
    }

    fn read_can_chl_status(&self, context: &ZChannelContext) -> Result<ZCanChlStatus, CanError> {
        let mut status: ZCanChlStatus = Default::default();
        match unsafe { (self.ZCAN_ReadChannelStatus)(context.channel_handler()?, &mut status) } as u32 {
            Self::STATUS_OK => Ok(status),
            code => Err(CanError::OtherError(format!("`ZCAN_ReadChannelStatus` ret: {}", code))),
        }
    }

    fn read_can_chl_error(&self, context: &ZChannelContext) -> Result<ZCanChlError, CanError> {
        let mut info: ZCanChlError = ZCanChlError::from(ZCanChlErrorV2::default());
        match unsafe { (self.ZCAN_ReadChannelErrInfo)(context.channel_handler()?, &mut info) } as u32  {
            Self::STATUS_OK => Ok(info),
            code => Err(CanError::OtherError(format!("`ZCAN_ReadChannelErrInfo` ret: {}", code))),
        }
    }

    fn clear_can_buffer(&self, context: &ZChannelContext) -> Result<(), CanError> {
        match unsafe { (self.ZCAN_ClearBuffer)(context.channel_handler()?) } as u32 {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::OtherError(format!("`ZCAN_ClearBuffer` ret: {}", code))),
        }
    }

    fn get_can_num(&self, context: &ZChannelContext, can_type: ZCanFrameType) -> Result<u32, CanError> {
        let ret = unsafe { (self.ZCAN_GetReceiveNum)(context.channel_handler()?, can_type as u8) };
        if ret > 0 {
            log::trace!("ZLGCAN - get receive {} number: {}.", can_type, ret);
        }
        Ok(ret as u32)
    }

    fn receive_can(&self, context: &ZChannelContext, size: u32, timeout: u32, resize: impl Fn(&mut Vec<Self::Frame>, usize)) -> Result<Vec<Self::Frame>, CanError> {
        let mut frames = Vec::new();
        resize(&mut frames, size as usize);

        let ret = unsafe { (self.ZCAN_Receive)(context.channel_handler()?, frames.as_mut_ptr(), size, timeout) };
        let ret = ret as u32;
        if ret < size {
            log::warn!("ZLGCAN - receive CAN frame expect: {}, actual: {}!", size, ret);
        }
        else if ret > 0 {
            log::trace!("ZLGCAN - receive CAN frame: {}", ret);
        }
        Ok(frames)
    }

    fn transmit_can(&self, context: &ZChannelContext, frames: Vec<Self::Frame>) -> Result<u32, CanError> {
        let len = frames.len() as u32;
        let ret = unsafe { (self.ZCAN_Transmit)(context.channel_handler()?, frames.as_ptr(), len) };
        let ret = ret as u32;
        if ret < len {
            log::warn!("ZLGCAN - transmit CAN frame expect: {}, actual: {}!", len, ret);
        }
        else {
            log::trace!("ZLGCAN - transmit CAN frame: {}", ret);
        }
        Ok(ret)
    }
}

impl ZLinApi for USBCANEApi<'_> {}
impl ZCloudApi for USBCANEApi<'_> {}

#[cfg(test)]
mod tests {
    use std::ffi::CString;
    use dlopen2::symbor::{Library, SymBorApi};
    // use crate::can::{CanChlCfgFactory, ZCanChlMode, ZCanChlType};
    use crate::device::{ZCanDeviceType, ZDeviceInfo};
    // use crate::api::ZDeviceApi;
    use super::USBCANEApi;

    #[test]
    fn usbcan_4e_u() {
        let dev_type = ZCanDeviceType::ZCAN_USBCAN_4E_U;
        let dev_idx = 0;
        let so_path = "library/linux/x86_64/libusbcan-4e.so";

        let lib = Library::open(so_path).expect("ZLGCAN - could not open library");
        let mut handlers = Vec::new();

        unsafe {
            let api = USBCANEApi::load(&lib).expect("ZLGCAN - could not load symbols!");
            let dev_hdl = (api.ZCAN_OpenDevice)(dev_type as u32, dev_idx, 0);
            if dev_hdl == USBCANEApi::INVALID_DEVICE_HANDLE {
                println!("Can't open the device!");
                return;
            }
            let mut dev_info = ZDeviceInfo::default();
            let ret = (api.ZCAN_GetDeviceInf)(dev_hdl, &mut dev_info);
            if ret != USBCANEApi::STATUS_OK {
                println!("Can't get the device info!");
                return;
            }

            let p = (api.GetIProperty)(dev_hdl);
            if p.is_null() {
                println!("Get property failed!");
                return;
            }
            let func = (*p).SetValue.expect("Can't get SetValue function!");

            for chl in 0..4 {
                let chl_hdl = (api.ZCAN_InitCAN)(dev_hdl, chl, std::ptr::null());
                if chl_hdl == USBCANEApi::INVALID_CHANNEL_HANDLE {
                    println!("Init channel: {} failed!", chl);
                    break;
                }
                handlers.push(chl_hdl);
                let ret = (api.ZCAN_ResetCAN)(chl_hdl);
                if ret != USBCANEApi::STATUS_OK {
                    println!("Reset channel: {} failed!", chl);
                }

                let path = CString::new(format!("info/channel/channel_{}/baud_rate", chl)).unwrap();
                let bitrate = CString::new(500_000.to_string()).unwrap();

                // let func = (*p).SetValue.expect("Can't get SetValue function!");
                let ret = func(path.as_ptr(), bitrate.as_ptr());

                if ret as u32 != USBCANEApi::STATUS_OK {
                    println!("SetValue failed: {}!", ret);
                    break;
                }

                let ret = (api.ZCAN_StartCAN)(chl_hdl);
                if ret as u32 != USBCANEApi::STATUS_OK {
                    println!("ZCAN_StartCAN failed!");
                    break;
                }
            }

            let ret = (api.ReleaseIProperty)(p);
            if ret as u32 != USBCANEApi::STATUS_OK {
                println!("ReleaseIProperty failed!");
            }

            for handler in handlers {
                let ret = (api.ZCAN_ResetCAN)(handler);
                if ret as u32 != USBCANEApi::STATUS_OK {
                    println!("ZCAN_ResetCAN failed!");
                }
            }

            let ret = (api.ZCAN_CloseDevice)(dev_hdl);
            if ret as u32 != USBCANEApi::STATUS_OK {
                println!("ZCAN_CloseDevice failed!");
            }
        }
    }
}
