#![allow(unused)]

#[cfg(target_os = "linux")]
pub(crate) mod linux;
#[cfg(target_os = "windows")]
pub(crate) mod windows;

use std::ffi::{c_char, c_void};
use rs_can::CanError;
use crate::can::{CanChlCfg, CanMessage, ZCanChlError, ZCanChlStatus, ZCanFrameType};
use crate::cloud::{ZCloudGpsFrame, ZCloudServerInfo, ZCloudUserData};
use crate::device::{CmdPath, IProperty, ZChannelContext, ZDeviceContext, ZDeviceInfo};
use crate::lin::{ZLinChlCfg, ZLinFrame, ZLinPublish, ZLinPublishEx, ZLinSubscribe};

#[allow(unused_variables, dead_code)]
pub trait ZDeviceApi {
    fn open(&self, context: &mut ZDeviceContext) -> Result<(), CanError>;
    fn close(&self, context: &ZDeviceContext) -> Result<(), CanError>;
    fn read_device_info(&self, context: &ZDeviceContext) -> Result<ZDeviceInfo, CanError>;
    fn is_online(&self, context: &ZDeviceContext) -> Result<bool, CanError> {
        Err(CanError::NotSupportedError)
    }
    fn get_property(&self, context: &ZChannelContext) -> Result<IProperty, CanError> {
        Err(CanError::NotSupportedError)
    }
    fn release_property(&self, p: &IProperty) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
    fn set_reference(&self, context: &ZChannelContext, cmd_path: &CmdPath, value: *const c_void) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
    fn get_reference(&self, context: &ZChannelContext, cmd_path: &CmdPath, value: *mut c_void) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
    fn set_value(&self, context: &ZChannelContext, cmd_path: &CmdPath, value: *const c_void) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
    fn get_value(&self, context: &ZChannelContext, cmd_path: &CmdPath) -> Result<*const c_void, CanError> {
        Err(CanError::NotSupportedError)
    }
    fn set_values(&self, context: &ZChannelContext, values: Vec<(CmdPath, *const c_char)>) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
    fn get_values(&self, context: &ZChannelContext, paths: Vec<CmdPath>) -> Result<Vec<String>, CanError> {
        Err(CanError::NotSupportedError)
    }
    fn debug(&self, level: u32) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
}

#[allow(unused_variables)]
pub trait ZCanApi {
    fn init_can_chl(&self, context: &mut ZChannelContext, cfg: &CanChlCfg) -> Result<(), CanError>;
    fn reset_can_chl(&self, context: &ZChannelContext) -> Result<(), CanError>;
    fn read_can_chl_status(&self, context: &ZChannelContext) -> Result<ZCanChlStatus, CanError>;
    fn read_can_chl_error(&self, context: &ZChannelContext) -> Result<ZCanChlError, CanError>;
    fn clear_can_buffer(&self, context: &ZChannelContext) -> Result<(), CanError>;
    fn get_can_num(&self, context: &ZChannelContext, can_type: ZCanFrameType) -> Result<u32, CanError>;
    fn receive_can(&self, context: &ZChannelContext, size: u32, timeout: u32) -> Result<Vec<CanMessage>, CanError>;
    fn transmit_can(&self, context: &ZChannelContext, frames: Vec<CanMessage>) -> Result<u32, CanError>;
    fn receive_canfd(&self, context: &ZChannelContext, size: u32, timeout: u32) -> Result<Vec<CanMessage>, CanError> {
        Err(CanError::NotSupportedError)
    }
    fn transmit_canfd(&self, context: &ZChannelContext, frames: Vec<CanMessage>) -> Result<u32, CanError> {
        Err(CanError::NotSupportedError)
    }
}

#[allow(unused_variables, dead_code)]
pub trait ZLinApi {
    fn init_lin_chl(&self, context: &mut ZChannelContext, cfg: &ZLinChlCfg) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
    fn reset_lin_chl(&self, context: &ZChannelContext) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
    fn clear_lin_buffer(&self, context: &ZChannelContext) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
    fn get_lin_num(&self, context: &ZChannelContext) -> Result<u32, CanError> {
        Err(CanError::NotSupportedError)
    }
    fn receive_lin(
        &self,
        context: &ZChannelContext,
        size: u32,
        timeout: u32,
    ) -> Result<Vec<ZLinFrame>, CanError> {
        Err(CanError::NotSupportedError)
    }
    fn transmit_lin(&self, context: &ZChannelContext, frames: Vec<ZLinFrame>) -> Result<u32, CanError> {
        Err(CanError::NotSupportedError)
    }
    fn set_lin_subscribe(&self, context: &ZChannelContext, cfg: Vec<ZLinSubscribe>)-> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
    fn set_lin_publish(&self, context: &ZChannelContext, cfg: Vec<ZLinPublish>) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
    fn wakeup_lin(&self, context: &ZChannelContext) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
    fn set_lin_publish_ex(&self, context: &ZChannelContext, cfg: Vec<ZLinPublishEx>) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
    #[deprecated(since="0.1.0", note="This method is deprecated!")]
    fn set_lin_slave_msg(&self, context: &ZChannelContext, msg: Vec<ZLinFrame>) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
    #[deprecated(since="0.1.0", note="This method is deprecated!")]
    fn clear_lin_slave_msg(&self, context: &ZChannelContext, pids: Vec<u8>) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
}

#[allow(unused_variables, dead_code)]
pub trait ZCloudApi {
    fn set_server(&self, server: ZCloudServerInfo) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
    fn connect_server(&self, username: &str, password: &str) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
    fn is_connected_server(&self) -> Result<bool, CanError> {
        Err(CanError::NotSupportedError)
    }
    fn disconnect_server(&self) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
    fn get_userdata(&self, update: i32) -> Result<ZCloudUserData, CanError> {
        Err(CanError::NotSupportedError)
    }
    fn receive_gps(
        &self,
        context: &ZDeviceContext,
        size: u32,
        timeout: u32
    ) -> Result<Vec<ZCloudGpsFrame>, CanError> {
        Err(CanError::NotSupportedError)
    }
}

