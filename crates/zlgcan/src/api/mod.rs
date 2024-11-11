#![allow(unused)]

#[cfg(target_os = "linux")]
pub(crate) mod linux;
#[cfg(target_os = "windows")]
pub(crate) mod windows;

use std::ffi::{c_char, c_void};
use crate::can::{CanChlCfg, ZCanChlError, ZCanChlStatus, ZCanFrameType};
use crate::cloud::{ZCloudGpsFrame, ZCloudServerInfo, ZCloudUserData};
use crate::device::{CmdPath, IProperty, ZChannelContext, ZDeviceContext, ZDeviceInfo};
use crate::error::ZCanError;
use crate::lin::{ZLinChlCfg, ZLinFrame, ZLinPublish, ZLinPublishEx, ZLinSubscribe};

#[allow(unused_variables, dead_code)]
pub trait ZDeviceApi {
    fn open(&self, context: &mut ZDeviceContext) -> Result<(), ZCanError>;
    fn close(&self, context: &ZDeviceContext) -> Result<(), ZCanError>;
    fn read_device_info(&self, context: &ZDeviceContext) -> Result<ZDeviceInfo, ZCanError>;
    fn is_online(&self, context: &ZDeviceContext) -> Result<bool, ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn get_property(&self, context: &ZChannelContext) -> Result<IProperty, ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn release_property(&self, p: &IProperty) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn set_reference(&self, context: &ZChannelContext, cmd_path: &CmdPath, value: *const c_void) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn get_reference(&self, context: &ZChannelContext, cmd_path: &CmdPath, value: *mut c_void) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn set_value(&self, context: &ZChannelContext, cmd_path: &CmdPath, value: *const c_void) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn get_value(&self, context: &ZChannelContext, cmd_path: &CmdPath) -> Result<*const c_void, ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn set_values(&self, context: &ZChannelContext, values: Vec<(CmdPath, *const c_char)>) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn get_values(&self, context: &ZChannelContext, paths: Vec<CmdPath>) -> Result<Vec<String>, ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn debug(&self, level: u32) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
}

#[allow(unused_variables)]
pub trait ZCanApi {
    type Frame;
    type FdFrame;
    fn init_can_chl(&self, context: &mut ZChannelContext, cfg: &CanChlCfg) -> Result<(), ZCanError>;
    fn reset_can_chl(&self, context: &ZChannelContext) -> Result<(), ZCanError>;
    fn read_can_chl_status(&self, context: &ZChannelContext) -> Result<ZCanChlStatus, ZCanError>;
    fn read_can_chl_error(&self, context: &ZChannelContext) -> Result<ZCanChlError, ZCanError>;
    fn clear_can_buffer(&self, context: &ZChannelContext) -> Result<(), ZCanError>;
    fn get_can_num(&self, context: &ZChannelContext, can_type: ZCanFrameType) -> Result<u32, ZCanError>;
    fn receive_can(&self, context: &ZChannelContext, size: u32, timeout: u32, resize: impl Fn(&mut Vec<Self::Frame>, usize)) -> Result<Vec<Self::Frame>, ZCanError>;
    fn transmit_can(&self, context: &ZChannelContext, frames: Vec<Self::Frame>) -> Result<u32, ZCanError>;
    fn receive_canfd(&self, context: &ZChannelContext, size: u32, timeout: u32, resize: fn(&mut Vec<Self::FdFrame>, usize)) -> Result<Vec<Self::FdFrame>, ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn transmit_canfd(&self, context: &ZChannelContext, frames: Vec<Self::FdFrame>) -> Result<u32, ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
}

#[allow(unused_variables, dead_code)]
pub trait ZLinApi {
    fn init_lin_chl(&self, context: &mut ZChannelContext, cfg: &ZLinChlCfg) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn reset_lin_chl(&self, context: &ZChannelContext) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn clear_lin_buffer(&self, context: &ZChannelContext) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn get_lin_num(&self, context: &ZChannelContext) -> Result<u32, ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn receive_lin(
        &self,
        context: &ZChannelContext,
        size: u32,
        timeout: u32,
        resize: impl Fn(&mut Vec<ZLinFrame>, usize)
    ) -> Result<Vec<ZLinFrame>, ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn transmit_lin(&self, context: &ZChannelContext, frames: Vec<ZLinFrame>) -> Result<u32, ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn set_lin_subscribe(&self, context: &ZChannelContext, cfg: Vec<ZLinSubscribe>)-> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn set_lin_publish(&self, context: &ZChannelContext, cfg: Vec<ZLinPublish>) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn wakeup_lin(&self, context: &ZChannelContext) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn set_lin_publish_ex(&self, context: &ZChannelContext, cfg: Vec<ZLinPublishEx>) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    #[deprecated(since="0.1.0", note="This method is deprecated!")]
    fn set_lin_slave_msg(&self, context: &ZChannelContext, msg: Vec<ZLinFrame>) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    #[deprecated(since="0.1.0", note="This method is deprecated!")]
    fn clear_lin_slave_msg(&self, context: &ZChannelContext, pids: Vec<u8>) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
}

#[allow(unused_variables, dead_code)]
pub trait ZCloudApi {
    fn set_server(&self, server: ZCloudServerInfo) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn connect_server(&self, username: &str, password: &str) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn is_connected_server(&self) -> Result<bool, ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn disconnect_server(&self) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn get_userdata(&self, update: i32) -> Result<ZCloudUserData, ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn receive_gps(
        &self,
        context: &ZDeviceContext,
        size: u32,
        timeout: u32,
        resize: impl Fn(&mut Vec<ZCloudGpsFrame>, usize)
    ) -> Result<Vec<ZCloudGpsFrame>, ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
}

