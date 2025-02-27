use std::ffi::{c_char, c_int, c_uchar, c_uint, c_ushort, c_void, CString};
use std::pin::Pin;
use rs_can::CanError;
use dlopen2::symbor::{Symbol, SymBorApi};
use crate::can::{CanChlCfg, ZCanChlError, ZCanChlErrorV2, ZCanChlStatus, ZCanChlType, ZCanFdFrameV2, ZCanFrameV3, ZCanFrameType, ZCanChlCfgV1};
use crate::cloud::{ZCloudGpsFrame, ZCloudServerInfo, ZCloudUserData};
use crate::device::{CmdPath, IProperty, ZCanDeviceType, ZChannelContext, ZDeviceContext, ZDeviceInfo};
use crate::lin::{ZLinChlCfg, ZLinFrame, ZLinPublish, ZLinPublishEx, ZLinSubscribe};
use crate::utils::c_str_to_string;

use crate::api::{ZCanApi, ZCloudApi, ZDeviceApi, ZLinApi};
use crate::constant::{STATUS_OFFLINE, STATUS_ONLINE, INTERNAL_RESISTANCE, PROTOCOL, CANFD_ABIT_BAUD_RATE, CANFD_DBIT_BAUD_RATE, BAUD_RATE, CLOCK};

#[allow(non_snake_case)]
#[derive(Debug, Clone, SymBorApi)]
pub(crate) struct Api<'a> {
    /// DEVICE_HANDLE FUNC_CALL ZCAN_OpenDevice(UINT device_type, UINT device_index, UINT reserved);
    ZCAN_OpenDevice: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_index: c_uint, reserved: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_CloseDevice(DEVICE_HANDLE device_handle);
    ZCAN_CloseDevice: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_GetDeviceInf(DEVICE_HANDLE device_handle, ZCAN_DEVICE_INFO *pInfo);
    ZCAN_GetDeviceInf: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint, dev_info: *const ZDeviceInfo) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_IsDeviceOnLine(DEVICE_HANDLE device_handle);
    ZCAN_IsDeviceOnLine: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint) -> c_uint>,

    /// UINT FUNC_CALL ZCAN_TransmitData(DEVICE_HANDLE device_handle, ZCANDataObj* pTransmit, UINT len);
    // ZCAN_TransmitData: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint, data: *const ZCANDataObj, len: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_ReceiveData(DEVICE_HANDLE device_handle, ZCANDataObj* pReceive, UINT len, int wait_time DEF(-1));
    // ZCAN_ReceiveData: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint, data: *mut ZCANDataObj, timeout: c_uint) -> c_uint>,

    /// UINT FUNC_CALL ZCAN_SetValue(DEVICE_HANDLE device_handle, const char* path, const void* value);
    ZCAN_SetValue: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint, path: *const c_char, value: *const c_void) -> c_uint>,
    /// const void* FUNC_CALL ZCAN_GetValue(DEVICE_HANDLE device_handle, const char* path);
    ZCAN_GetValue: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint, path: *const c_char) -> *const c_void>,
    /// IProperty* FUNC_CALL GetIProperty(DEVICE_HANDLE device_handle);
    GetIProperty: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint) -> *const IProperty>,
    /// UINT FUNC_CALL ReleaseIProperty(IProperty * pIProperty);
    ReleaseIProperty: Symbol<'a, unsafe extern "C" fn(p: *const IProperty) -> c_uint>,

    /// CHANNEL_HANDLE FUNC_CALL ZCAN_InitCAN(DEVICE_HANDLE device_handle, UINT can_index, ZCAN_CHANNEL_INIT_CONFIG* pInitConfig);
    ZCAN_InitCAN: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint, channel: c_uint, cfg: *const ZCanChlCfgV1) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_StartCAN(CHANNEL_HANDLE channel_handle);
    ZCAN_StartCAN: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_ResetCAN(CHANNEL_HANDLE channel_handle);
    ZCAN_ResetCAN: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_ClearBuffer(CHANNEL_HANDLE channel_handle);
    ZCAN_ClearBuffer: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_ReadChannelErrInfo(CHANNEL_HANDLE channel_handle, ZCAN_CHANNEL_ERR_INFO* pErrInfo);
    ZCAN_ReadChannelErrInfo: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, *mut ZCanChlError) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_ReadChannelStatus(CHANNEL_HANDLE channel_handle, ZCAN_CHANNEL_STATUS* pCANStatus);
    ZCAN_ReadChannelStatus: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, *mut ZCanChlStatus) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_GetReceiveNum(CHANNEL_HANDLE channel_handle, BYTE type);//type:TYPE_CAN, TYPE_CANFD, TYPE_ALL_DATA
    ZCAN_GetReceiveNum: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, can_type: c_uchar) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_Transmit(CHANNEL_HANDLE channel_handle, ZCAN_Transmit_Data* pTransmit, UINT len);
    ZCAN_Transmit: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, frames: *const ZCanFrameV3, len: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_Receive(CHANNEL_HANDLE channel_handle, ZCAN_Receive_Data* pReceive, UINT len, int wait_time DEF(-1));
    ZCAN_Receive: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, frames: *mut ZCanFrameV3, len: c_uint, timeout: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_TransmitFD(CHANNEL_HANDLE channel_handle, ZCAN_TransmitFD_Data* pTransmit, UINT len);
    ZCAN_TransmitFD: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, frames: *const ZCanFdFrameV2, len: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_ReceiveFD(CHANNEL_HANDLE channel_handle, ZCAN_ReceiveFD_Data* pReceive, UINT len, int wait_time DEF(-1));
    ZCAN_ReceiveFD: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, frames: *mut ZCanFdFrameV2, len: c_uint, timeout: c_uint) -> c_uint>,

    /// void FUNC_CALL ZCLOUD_SetServerInfo(const char* httpSvr, unsigned short httpPort, const char* authSvr, unsigned short authPort);
    ZCLOUD_SetServerInfo: Symbol<'a, unsafe extern "C" fn(http: *const c_char, port1: c_ushort, auth: *const c_char, port2: c_ushort)>,
    /// UINT FUNC_CALL ZCLOUD_ConnectServer(const char* username, const char* password); // return 0:success, 1:failure, 2:https error, 3:user login info error, 4:mqtt connection error, 5:no device
    ZCLOUD_ConnectServer: Symbol<'a, unsafe extern "C" fn(username: *const c_char, password: *const c_char) -> c_uint>,
    /// bool FUNC_CALL ZCLOUD_IsConnected();
    ZCLOUD_IsConnected: Symbol<'a, unsafe extern "C" fn() -> bool>,
    /// UINT FUNC_CALL ZCLOUD_DisconnectServer(); // return 0:success, 1:failure
    ZCLOUD_DisconnectServer: Symbol<'a, unsafe extern "C" fn() -> c_uint>,
    /// const ZCLOUD_USER_DATA* FUNC_CALL ZCLOUD_GetUserData(int update DEF(0));
    ZCLOUD_GetUserData: Symbol<'a, unsafe extern "C" fn(update: c_int) -> *const ZCloudUserData>,
    /// UINT FUNC_CALL ZCLOUD_ReceiveGPS(DEVICE_HANDLE device_handle, ZCLOUD_GPS_FRAME* pReceive, UINT len, int wait_time DEF(-1));
    ZCLOUD_ReceiveGPS: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint, frames: *mut ZCloudGpsFrame, len: c_uint, timeout: c_uint) -> c_uint>,

    /// CHANNEL_HANDLE FUNC_CALL ZCAN_InitLIN(DEVICE_HANDLE device_handle, UINT can_index, PZCAN_LIN_INIT_CONFIG pLINInitConfig);
    ZCAN_InitLIN: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint, channel: c_uint, cfg: *const ZLinChlCfg) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_StartLIN(CHANNEL_HANDLE channel_handle);
    ZCAN_StartLIN: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_ResetLIN(CHANNEL_HANDLE channel_handle);
    ZCAN_ResetLIN: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_TransmitLIN(CHANNEL_HANDLE channel_handle, PZCAN_LIN_MSG pSend, UINT Len);
    ZCAN_TransmitLIN: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, frames: *const ZLinFrame, len: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_GetLINReceiveNum(CHANNEL_HANDLE channel_handle);
    ZCAN_GetLINReceiveNum: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_ReceiveLIN(CHANNEL_HANDLE channel_handle, PZCAN_LIN_MSG pReceive, UINT Len,int WaitTime);
    ZCAN_ReceiveLIN: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, frames: *mut ZLinFrame, len: c_uint, timeout: c_uint) -> c_uint>,
    // UINT FUNC_CALL ZCAN_ClearLINBuffer(CHANNEL_HANDLE channel_handle);
    // ZCAN_ClearLINBuffer: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_SetLINSlaveMsg(CHANNEL_HANDLE channel_handle, PZCAN_LIN_MSG pSend, UINT nMsgCount);
    ZCAN_SetLINSlaveMsg: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, frames: *const ZLinFrame, size: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_ClearLINSlaveMsg(CHANNEL_HANDLE channel_handle, BYTE* pLINID, UINT nIDCount);
    ZCAN_ClearLINSlaveMsg: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, ids: *const c_uchar, size: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_WakeUpLIN(CHANNEL_HANDLE channel_handle);
    ZCAN_WakeUpLIN: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_SetLINSubscribe(CHANNEL_HANDLE channel_handle, PZCAN_LIN_SUBSCIBE_CFG pSend, UINT nSubscribeCount);
    ZCAN_SetLINSubscribe: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, cfg: *const ZLinSubscribe, len: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_SetLINPublish(CHANNEL_HANDLE channel_handle, PZCAN_LIN_PUBLISH_CFG pSend, UINT nPublishCount);
    ZCAN_SetLINPublish: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, cfg: *const ZLinPublish, len: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_SetLINPublishEx(CHANNEL_HANDLE channel_handle, PZCAN_LIN_PUBLISH_CFG_EX pSend, UINT nPublishCount);
    ZCAN_SetLINPublishEx: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, cfg: *const ZLinPublishEx, len: c_uint) -> c_uint>,

    // ZCAN_RET_STATUS FUNC_CALL ZCAN_UDS_ControlEX(DEVICE_HANDLE device_handle, ZCAN_UDS_DATA_DEF dataType,
    //                                              const ZCAN_UDS_CTRL_REQ *ctrl, ZCAN_UDS_CTRL_RESP *resp);
    // ZCAN_UDS_ControlEX: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, data_type: ZCAN_UDS_DATA_DEF, ctrl: *const ZCAN_UDS_CTRL_REQ, resp: *mut ZCAN_UDS_CTRL_RESP) -> c_uint>,
    // ZCAN_RET_STATUS FUNC_CALL ZCAN_UDS_RequestEX(DEVICE_HANDLE device_handle, const ZCANUdsRequestDataObj *requestData,
    //                                              ZCAN_UDS_RESPONSE *resp, BYTE *dataBuf, UINT dataBufSize);
    // ZCAN_UDS_RequestEX: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, data: *const ZCANUdsRequestDataObj, resp: *mut ZCAN_UDS_CTRL_RESP, buff: *mut c_uchar, buff_size: c_uint) -> c_uint>,
    // ZCAN_RET_STATUS FUNC_CALL ZCAN_UDS_Control(DEVICE_HANDLE device_handle, const ZCAN_UDS_CTRL_REQ *ctrl, ZCAN_UDS_CTRL_RESP *resp);
    // ZCAN_UDS_Control: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, ctrl: *const ZCAN_UDS_CTRL_REQ, resp: *mut ZCAN_UDS_CTRL_RESP) -> c_uint>,
    // ZCAN_RET_STATUS FUNC_CALL ZCAN_UDS_Request(DEVICE_HANDLE device_handle, const ZCAN_UDS_REQUEST *req,
    //                                            ZCAN_UDS_RESPONSE *resp, BYTE *dataBuf, UINT dataBufSize);
    // ZCAN_UDS_Request: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, req: *const ZCAN_UDS_REQUEST, resp: *mut ZCAN_UDS_RESPONSE, buff: *mut c_uchar, buff_size: c_uint) -> c_uint>,
}

impl Api<'_> {
    const INVALID_DEVICE_HANDLE: u32 = 0;
    const INVALID_CHANNEL_HANDLE: u32 = 0;
    const STATUS_OK: u32 = 1;
}

impl ZDeviceApi for Api<'_> {
    fn open(&self, context: &mut ZDeviceContext) -> Result<(), CanError> {
        match unsafe { (self.ZCAN_OpenDevice)(context.device_type() as u32, context.device_index(), 0) } {
            Self::INVALID_DEVICE_HANDLE => Err(CanError::OtherError(format!("`ZCAN_OpenDevice` ret = {}", Self::INVALID_DEVICE_HANDLE))),
            v => {
                context.set_device_handler(v);
                Ok(())
            },
        }
    }
    fn close(&self, context: &ZDeviceContext) -> Result<(), CanError> {
        match unsafe { (self.ZCAN_CloseDevice)(context.device_handler()?) } {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::OtherError(format!("`ZCAN_CloseDevice` ret = {}", code))),
        }
    }
    fn read_device_info(&self, context: &ZDeviceContext) -> Result<ZDeviceInfo, CanError> {
        let mut info = ZDeviceInfo::default();
        match unsafe { (self.ZCAN_GetDeviceInf)(context.device_handler()?, &mut info) } {
            Self::STATUS_OK => Ok(info),
            code => Err(CanError::OtherError(format!("`ZCAN_GetDeviceInf` ret = {}", code))),
        }
    }
    fn is_online(&self, context: &ZDeviceContext) -> Result<bool, CanError> {
        unsafe {
            match (self.ZCAN_IsDeviceOnLine)(context.device_handler()?) {
                STATUS_ONLINE => Ok(true),
                STATUS_OFFLINE => Ok(false),
                code => Err(CanError::OtherError(format!("`ZCAN_IsDeviceOnLine` ret = {}", code))),
            }
        }
    }
    fn get_property(&self, context: &ZChannelContext) -> Result<IProperty, CanError> {
        unsafe {
            let ret = (self.GetIProperty)(context.device_handler()?);
            if ret.is_null() {
                Err(CanError::OtherError(format!("`GetIProperty` ret = {}", 0)))
            } else {
                Ok(*ret)
            }
        }
    }
    fn release_property(&self, p: &IProperty) -> Result<(), CanError> {
        unsafe {
            match (self.ReleaseIProperty)(p) {
                Self::STATUS_OK => Ok(()),
                code => Err(CanError::OtherError(format!("`ReleaseIProperty` ret = {}", code))),
            }
        }
    }
    fn get_value(&self, context: &ZChannelContext, cmd_path: &CmdPath) -> Result<*const c_void, CanError> {
        unsafe {
            let path = cmd_path.get_path();
            let path = CString::new(path).map_err(|e| CanError::OtherError(e.to_string()))?;
            if context.device_type().get_value_support() {
                let ret = (self.ZCAN_GetValue)(context.device_handler()?, path.as_ptr() as *const c_char);
                if ret.is_null() {
                    Err(CanError::OtherError(format!("`ZCAN_GetValue` ret = {}", 0)))
                } else {
                    Ok(ret)
                }
            } else {
                Err(CanError::OtherError("method not supported".to_owned()))
            }
        }
    }
    fn set_value(&self, context: &ZChannelContext, cmd_path: &CmdPath, value: *const c_void) -> Result<(), CanError> {
        unsafe {
            let path = cmd_path.get_path();
            let _path = CString::new(path).map_err(|e| CanError::OtherError(e.to_string()))?;
            // let _value = CString::new(value).map_err(|e| CanError::CStringConvertFailed(e.to_string()))?;
            match (self.ZCAN_SetValue)(context.device_handler()?, _path.as_ptr() as *const c_char, value) {
                Self::STATUS_OK => Ok(()),
                code=> Err(CanError::OtherError(format!("`ZCAN_SetValue` ret = {}", code))),
            }
        }
    }
    fn set_values(&self, context: &ZChannelContext, values: Vec<(CmdPath, *const c_char)>) -> Result<(), CanError> {
        unsafe {
            let p = self.get_property(context)?;
            match p.SetValue {
                Some(f) => {
                    for (cmd, value) in values {
                        let path = cmd.get_path();
                        // let _path = format!("{}/{}", path, channel);
                        let _path = CString::new(path).map_err(|e| CanError::OtherError(e.to_string()))?;
                        match f(_path.as_ptr(), value) {
                            1 => (),
                            _ => log::warn!("ZLGCAN - set `{}` failed!", path),
                        }
                    }

                    let _ = self.release_property(&p).is_err_and(|e| -> bool {
                        log::warn!("{}", e);
                        true
                    });
                    Ok(())
                },
                None => Err(CanError::OtherError("method not supported".to_owned())),
            }
        }
    }
    fn get_values(&self, context: &ZChannelContext, paths: Vec<CmdPath>) -> Result<Vec<String>, CanError> {
        unsafe {
            let p = self.get_property(context)?;
            let channel = context.channel();
            match p.GetValue {
                Some(f) => {
                    let mut result = Vec::new();
                    for cmd in paths {
                        let path = cmd.get_path();
                        let _path = CString::new(format!("{}/{}", path, channel)).map_err(|e| CanError::OtherError(e.to_string()))?;
                        let ret = f(_path.as_ptr());
                        let v = c_str_to_string(ret)?;
                        result.push(v);
                    }

                    let _ = self.release_property(&p).is_err_and(|e| -> bool {
                        log::warn!("{}", e);
                        true
                    });

                    Ok(result)
                },
                None => Err(CanError::OtherError("method not supported".to_owned())),
            }
        }
    }
}

impl ZCanApi for Api<'_> {
    type Frame = ZCanFrameV3;
    type FdFrame = ZCanFdFrameV2;
    fn init_can_chl(&self, context: &mut ZChannelContext, cfg: &CanChlCfg) -> Result<(), CanError> {
        let dev_type = context.device_type();
        let channel = context.channel();
        unsafe {
            if !matches!(dev_type, ZCanDeviceType::ZCAN_USBCAN1 | ZCanDeviceType::ZCAN_USBCAN2) {
                // configure the clock
                if let Some(clock) = cfg.clock() {
                    let clock_path = CmdPath::new_path(CLOCK);
                    let value = CString::new(clock.to_string()).map_err(|e| CanError::OtherError(e.to_string()))?;
                    self.set_value(context, &clock_path, value.as_ptr() as *const c_void)?;
                }
                // set channel resistance status
                if dev_type.has_resistance() {
                    let state = (cfg.extra().resistance() as u32).to_string();
                    let resistance_path = format!("{}/{}", channel, INTERNAL_RESISTANCE);
                    let resistance_path = CmdPath::new_path(resistance_path.as_str());
                    let value = CString::new(state).map_err(|e| CanError::OtherError(e.to_string()))?;
                    self.set_value(context, &resistance_path, value.as_ptr() as *const c_void)?;
                }
                // set channel protocol
                let can_type = cfg.can_type()?;
                let protocol = (can_type as u32).to_string();
                let protocol_path = format!("{}/{}", channel, PROTOCOL);
                let protocol_path = CmdPath::new_path(protocol_path.as_str());
                let value = CString::new(protocol).map_err(|e| CanError::OtherError(e.to_string()))?;
                self.set_value(context, &protocol_path, value.as_ptr() as *const c_void)?;

                // set channel bitrate
                let bitrate = cfg.bitrate();
                if dev_type.canfd_support() {
                    let abitrate_path = format!("{}/{}", channel, CANFD_ABIT_BAUD_RATE);
                    let abitrate_path = CmdPath::new_path(abitrate_path.as_str());
                    let value = CString::new(bitrate.to_string()).map_err(|e| CanError::OtherError(e.to_string()))?;
                    self.set_value(context, &abitrate_path, value.as_ptr() as *const c_void)?;
                    match can_type {
                        ZCanChlType::CANFD_ISO | ZCanChlType::CANFD_NON_ISO => {
                            let dbitrate = cfg.extra().dbitrate().unwrap_or(bitrate).to_string();
                            let dbitrate_path = format!("{}/{}", channel, CANFD_DBIT_BAUD_RATE);
                            let dbitrate_path = CmdPath::new_path(dbitrate_path.as_str());
                            let value = CString::new(dbitrate).map_err(|e| CanError::OtherError(e.to_string()))?;
                            self.set_value(context, &dbitrate_path, value.as_ptr() as *const c_void)?;
                        },
                        _ => {},
                    }
                }
                else {
                    let bitrate_path = format!("{}/{}", channel, BAUD_RATE);
                    let bitrate_path = CmdPath::new_path(bitrate_path.as_str());
                    let value = CString::new(bitrate.to_string()).map_err(|e| CanError::OtherError(e.to_string()))?;
                    self.set_value(context, &bitrate_path, value.as_ptr() as *const c_void)?;
                }
            }

            let cfg = ZCanChlCfgV1::try_from(cfg)?;
            match (self.ZCAN_InitCAN)(context.device_handler()?, channel as u32, &cfg) {
                Self::INVALID_CHANNEL_HANDLE => Err(CanError::OtherError(format!("`ZCAN_InitCAN` ret = {}", Self::INVALID_CHANNEL_HANDLE))),
                handler => match (self.ZCAN_StartCAN)(handler) {
                    Self::STATUS_OK => {
                        context.set_channel_handler(Some(handler));
                        Ok(())
                    },
                    code => Err(CanError::OtherError(format!("`ZCAN_StartCAN` ret = {}", code))),
                }
            }
        }
    }

    fn reset_can_chl(&self, context: &ZChannelContext) -> Result<(), CanError> {
        match unsafe { (self.ZCAN_ResetCAN)(context.channel_handler()?) } {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::OtherError(format!("`ZCAN_ResetCAN` ret = {}", code))),
        }
    }

    fn read_can_chl_status(&self, context: &ZChannelContext) -> Result<ZCanChlStatus, CanError> {
        let mut status: ZCanChlStatus = Default::default();
        match unsafe { (self.ZCAN_ReadChannelStatus)(context.channel_handler()?, &mut status) } {
            Self::STATUS_OK => Ok(status),
            code => Err(CanError::OtherError(format!("`ZCAN_ReadChannelStatus` ret = {}", code))),
        }
    }

    fn read_can_chl_error(&self, context: &ZChannelContext) -> Result<ZCanChlError, CanError> {
        let mut info: ZCanChlError = ZCanChlError::from(ZCanChlErrorV2::default());
        match unsafe { (self.ZCAN_ReadChannelErrInfo)(context.channel_handler()?, &mut info) } {
            Self::STATUS_OK => Ok(info),
            code => Err(CanError::OtherError(format!("`ZCAN_ReadChannelErrInfo` ret = {}", code))),
        }
    }

    fn clear_can_buffer(&self, context: &ZChannelContext) -> Result<(), CanError> {
        match unsafe { (self.ZCAN_ClearBuffer)(context.channel_handler()?) } {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::OtherError(format!("`ZCAN_ClearBuffer` ret = {}", code))),
        }
    }

    fn get_can_num(&self, context: &ZChannelContext, can_type: ZCanFrameType) -> Result<u32, CanError> {
        let ret = unsafe { (self.ZCAN_GetReceiveNum)(context.channel_handler()?, can_type as u8) };
        if ret > 0 {
            log::trace!("ZLGCAN - get receive {} number: {}.", can_type, ret);
        }
        Ok(ret)
    }

    fn receive_can(&self, context: &ZChannelContext, size: u32, timeout: u32, resize: impl Fn(&mut Vec<Self::Frame>, usize)) -> Result<Vec<Self::Frame>, CanError> {
        let mut frames = Vec::new();

        let mut count = 0;
        for _ in 0..size {
            let mut frame: ZCanFrameV3 = Default::default();
            let ret = unsafe { (self.ZCAN_Receive)(context.channel_handler()?, &mut frame, 1, timeout) };
            if ret == 1 {
                count += 1;
                frames.push(frame);
            }
        }
        if count < size {
            log::warn!("ZLGCAN - receive CAN frame expect: {}, actual: {}!", size, count);
        }

        Ok(frames)
    }

    fn transmit_can(&self, context: &ZChannelContext, frames: Vec<Self::Frame>) -> Result<u32, CanError> {
        let len = frames.len() as u32;
        // method 1
        // let ret = unsafe { (self.ZCAN_Transmit)(chl_hdl, frames.as_ptr(), len) };
        // if ret < len {
        //     log::warn!("ZLGCAN - transmit CAN frame expect: {}, actual: {}!", len, ret);
        // }
        // Ok(ret)
        // method 2
        // let mut boxed_slice: Box<[ZCanFrame]> = frames.into_boxed_slice();
        // let array: *mut ZCanFrame = boxed_slice.as_mut_ptr();
        // // let ptr = frames.as_ptr();
        // let ret = unsafe { (self.ZCAN_Transmit)(chl_hdl, array, len) };
        // std::mem::forget(boxed_slice);
        // if ret < len {
        //     log::warn!("ZLGCAN - transmit CAN frame expect: {}, actual: {}!", len, ret);
        // }
        // Ok(ret)
        // method 3: just do like this because of pointer offset TODO
        let mut count = 0;
        let chl_hdl = context.channel_handler()?;
        frames.iter().for_each(|frame| {
            let ret = unsafe { (self.ZCAN_Transmit)(chl_hdl, frame, 1) };
            count += ret;
        });
        if count < len {
            log::warn!("ZLGCAN - transmit CAN frame expect: {}, actual: {}!", len, count);
        }
        else {
            log::trace!("ZLGCAN - transmit CAN frame: {}", count);
        }
        Ok(count)
    }

    fn receive_canfd(&self, context: &ZChannelContext, size: u32, timeout: u32, resize: fn(&mut Vec<Self::FdFrame>, usize)) -> Result<Vec<Self::FdFrame>, CanError> {
        let mut frames = Vec::new();
        // frames.resize_with(size as usize, Default::default);

        let mut count = 0;
        for _ in 0..size {
            let mut frame: ZCanFdFrameV2 = Default::default();
            let ret = unsafe { (self.ZCAN_ReceiveFD)(context.channel_handler()?, &mut frame, 1, timeout) };
            if ret == 1 {
                count += 1;
                frames.push(frame);
            }
        }
        if count < size {
            log::warn!("ZLGCAN - receive CANFD frame expect: {}, actual: {}!", size, count);
        }

        Ok(frames)
    }

    fn transmit_canfd(&self, context: &ZChannelContext, frames: Vec<Self::FdFrame>) -> Result<u32, CanError> {
        let len = frames.len() as u32;
        // let ret = unsafe { (self.ZCAN_TransmitFD)(chl_hdl, frames.as_ptr(), len) };
        // if ret < len {
        //     warn!("ZLGCAN - transmit CANFD frame expect: {}, actual: {}!", len, ret);
        // }
        // Ok(ret)
        let mut count = 0;
        let chl_hdl = context.channel_handler()?;
        frames.iter().for_each(|frame| {
            let ret = unsafe { (self.ZCAN_TransmitFD)(chl_hdl, frame, 1) };
            count += ret;
        });
        if count < len {
            log::warn!("ZLGCAN - transmit CAN-FD frame expect: {}, actual: {}!", len, count);
        }
        else {
            log::trace!("ZLGCAN - transmit CAN-FD frame: {}", count);
        }
        Ok(count)
    }
}

impl ZLinApi for Api<'_> {
    fn init_lin_chl(&self, context: &mut ZChannelContext, cfg: &ZLinChlCfg) -> Result<(), CanError> {
        unsafe {
            let dev_hdl = context.device_handler()?;
            let channel = context.channel();
            match (self.ZCAN_InitLIN)(dev_hdl, channel as u32, cfg) {
                Self::INVALID_CHANNEL_HANDLE => Err(CanError::OtherError(format!("`ZCAN_InitLIN` ret = {}", Self::INVALID_CHANNEL_HANDLE))),
                handler => match (self.ZCAN_StartLIN)(dev_hdl) {
                    Self::STATUS_OK => {
                        context.set_channel_handler(Some(handler));
                        Ok(())
                    },
                    code => Err(CanError::OtherError(format!("`ZCAN_StartLIN` ret = {}", code))),
                }
            }
        }
    }
    fn reset_lin_chl(&self, context: &ZChannelContext) -> Result<(), CanError> {
        match unsafe { (self.ZCAN_ResetLIN)(context.channel_handler()?) } {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::OtherError(format!("`ZCAN_ResetLIN` ret = {}", code))),
        }
    }
    fn get_lin_num(&self, context: &ZChannelContext) -> Result<u32, CanError> {
        let ret = unsafe { (self.ZCAN_GetLINReceiveNum)(context.channel_handler()?) };
        if ret > 0 {
            log::trace!("ZLGCAN - get receive LIN number: {}.", ret);
        }
        Ok(ret)
    }
    fn receive_lin(&self, context: &ZChannelContext, size: u32, timeout: u32, resize: impl Fn(&mut Vec<ZLinFrame>, usize)) -> Result<Vec<ZLinFrame>, CanError> {
        let mut frames = Vec::new();

        resize(&mut frames, size as usize);

        let ret = unsafe { (self.ZCAN_ReceiveLIN)(context.channel_handler()?, frames.as_mut_ptr(), size, timeout) };
        if ret < size {
            log::warn!("ZLGCAN - receive LIN frame expect: {}, actual: {}!", size, ret);
        }
        else if ret > 0 {
            log::trace!("ZLGCAN - receive LIN frame: {}", ret);
        }
        Ok(frames)
    }
    fn transmit_lin(&self, context: &ZChannelContext, frames: Vec<ZLinFrame>) -> Result<u32, CanError> {
        let len = frames.len() as u32;
        let ret = unsafe { (self.ZCAN_TransmitLIN)(context.channel_handler()?, frames.as_ptr(), len) };
        if ret < len {
            log::warn!("ZLGCAN - transmit LIN frame expect: {}, actual: {}!", len, ret);
        }
        else {
            log::trace!("ZLGCAN - transmit LIN frame: {}", ret);
        }
        Ok(ret)
    }
    fn set_lin_subscribe(&self, context: &ZChannelContext, cfg: Vec<ZLinSubscribe>) -> Result<(), CanError> {
        let len = cfg.len() as u32;
        match unsafe { (self.ZCAN_SetLINSubscribe)(context.channel_handler()?, cfg.as_ptr(), len) } {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::OtherError(format!("`ZCAN_SetLINSubscribe` ret = {}", code))),
        }
    }
    fn set_lin_publish(&self, context: &ZChannelContext, cfg: Vec<ZLinPublish>) -> Result<(), CanError> {
        let len = cfg.len() as u32;
        match unsafe { (self.ZCAN_SetLINPublish)(context.channel_handler()?, cfg.as_ptr(), len) } {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::OtherError(format!("`ZCAN_SetLINPublish` ret = {}", code))),
        }
    }
    fn wakeup_lin(&self, context: &ZChannelContext) -> Result<(), CanError> {
        match unsafe { (self.ZCAN_WakeUpLIN)(context.channel_handler()?) } {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::OtherError(format!("`ZCAN_WakeUpLIN` ret = {}", code))),
        }
    }
    fn set_lin_publish_ex(&self, context: &ZChannelContext, cfg: Vec<ZLinPublishEx>) -> Result<(), CanError> {
        let len = cfg.len() as u32;
        match unsafe { (self.ZCAN_SetLINPublishEx)(context.channel_handler()?, cfg.as_ptr(), len) } {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::OtherError(format!("`ZCAN_SetLINPublishEx` ret = {}", code))),
        }
    }
    fn set_lin_slave_msg(&self, context: &ZChannelContext, msg: Vec<ZLinFrame>) -> Result<(), CanError> {
        let len = msg.len() as u32;
        match unsafe { (self.ZCAN_SetLINSlaveMsg)(context.channel_handler()?, msg.as_ptr(), len) } {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::OtherError(format!("`ZCAN_SetLINSlaveMsg` ret = {}", code))),
        }
    }
    fn clear_lin_slave_msg(&self, context: &ZChannelContext, pids: Vec<u8>) -> Result<(), CanError> {
        let len = pids.len() as u32;
        match unsafe { (self.ZCAN_ClearLINSlaveMsg)(context.channel_handler()?, pids.as_ptr(), len) } {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::OtherError(format!("`ZCAN_ClearLINSlaveMsg` ret = {}", code))),
        }
    }
}

impl ZCloudApi for Api<'_> {
    fn set_server(&self, server: ZCloudServerInfo) -> Result<(), CanError> {
        unsafe { (self.ZCLOUD_SetServerInfo)(server.http_url, server.http_port, server.mqtt_url, server.mqtt_port) }

        Ok(())
    }
    fn connect_server(&self, username: &str, password: &str) -> Result<(), CanError> {
        let username = CString::new(username).map_err(|e| CanError::OtherError(e.to_string()))?;
        let password = CString::new(password).map_err(|e| CanError::OtherError(e.to_string()))?;
        match unsafe { (self.ZCLOUD_ConnectServer)(username.as_ptr(), password.as_ptr()) } {
            Self::STATUS_OK => Ok(()),
            code=> Err(CanError::OtherError(format!("`ZCLOUD_ConnectServer` ret = {}", code))),
        }
    }
    fn is_connected_server(&self) -> Result<bool, CanError> {
        unsafe { Ok((self.ZCLOUD_IsConnected)()) }
    }
    fn disconnect_server(&self) -> Result<(), CanError> {
        match unsafe { (self.ZCLOUD_DisconnectServer)() } {
            0 => Ok(()),
            code=> Err(CanError::OtherError(format!("`ZCLOUD_DisconnectServer` ret = {}", code))),
        }
    }
    fn get_userdata(&self, update: i32) -> Result<ZCloudUserData, CanError> {
        unsafe {
            let data = (self.ZCLOUD_GetUserData)(update);
            if data.is_null() {
                Err(CanError::OtherError(format!("`ZCLOUD_GetUserData` ret = {}", 0)))
            }
            else {
                Ok(*data)
            }
        }
    }
    fn receive_gps(&self, context: &ZDeviceContext, size: u32, timeout: u32, resize: impl Fn(&mut Vec<ZCloudGpsFrame>, usize)) -> Result<Vec<ZCloudGpsFrame>, CanError> {
        let mut frames = Vec::new();
        resize(&mut frames, size as usize);

        let ret = unsafe { (self.ZCLOUD_ReceiveGPS)(context.device_handler()?, frames.as_mut_ptr(), size, timeout) };
        if ret < size {
            log::warn!("ZLGCAN - receive GPS frame expect: {}, actual: {}!", size, ret);
        }
        else if ret > 0 {
            log::trace!("ZLGCAN - receive GPS frame: {}", ret);
        }
        Ok(frames)
    }
}

#[cfg(test)]
mod tests {
    use dlopen2::symbor::{Library, SymBorApi};
    // use crate::device::ZCanDeviceType;
    use super::Api;

    #[test]
    fn load_symbols() {
        // let dev_type = ZCanDeviceType::ZCAN_USBCAN1;

        let dll_path = "library/windows/x86_64/zlgcan.dll";
        let lib = Library::open(dll_path).expect("ZLGCAN - could not open library");

        let _ = unsafe { Api::load(&lib) }.expect("ZLGCAN - could not load symbols!");
    }
}

