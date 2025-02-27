use rs_can::{CanDriver, CanError, Frame};
use crate::can::{CanChlCfg, CanMessage, ZCanChlError, ZCanChlStatus, ZCanFrameType};
use crate::cloud::{ZCloudGpsFrame, ZCloudServerInfo, ZCloudUserData};
use crate::device::{DeriveInfo, Handler, ZCanDeviceType, ZChannelContext, ZDeviceInfo};
use crate::lin::{ZLinChlCfg, ZLinFrame, ZLinPublish, ZLinPublishEx, ZLinSubscribe};

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::ZCanDriver;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::ZCanDriver;

impl CanDriver for ZCanDriver {
    type Channel = u8;
    type Frame = CanMessage;

    #[inline]
    fn opened_channels(&self) -> Vec<Self::Channel> {
        match &self.handler {
            Some(v) =>
                v.can_channels().keys()
                    .map(|v| v.clone())
                    .collect(),
            None => vec![],
        }
    }

    fn transmit(&self, msg: Self::Frame, _: Option<u32>) -> Result<(), CanError> {
        let channel = msg.channel();
        if msg.is_can_fd() {
            self.transmit_canfd(channel, vec![msg, ])?;
        }
        else {
            self.transmit_can(channel, vec![msg, ])?;
        }

        Ok(())
    }

    fn receive(&self, channel: Self::Channel, timeout: Option<u32>) -> Result<Vec<Self::Frame>, CanError> {
        let mut results: Vec<CanMessage> = Vec::new();

        let count_can = self.get_can_num(channel, ZCanFrameType::CAN)?;
        if count_can > 0 {
            log::trace!("RUST-CAN - received CAN: {}", count_can);
            let mut frames = self.receive_can(channel, count_can, timeout)?;
            results.append(&mut frames);
        }

        if self.device_type().canfd_support() {
            let count_fd = self.get_can_num(channel, ZCanFrameType::CANFD)?;
            if count_fd > 0 {
                log::trace!("RUST-CAN - received CANFD: {}", count_fd);
                let mut frames = self.receive_canfd(channel, count_fd, timeout)?;
                results.append(&mut frames);
            }
        }

        Ok(results)
    }

    #[inline]
    fn shutdown(&mut self) {
        self.close()
    }
}

#[allow(unused_variables)]
pub trait ZDevice {
    fn new(dev_type: u32, dev_idx: u32, derive: Option<DeriveInfo>) -> Result<Self, CanError>
        where Self: Sized;
    fn device_type(&self) -> ZCanDeviceType;
    fn device_index(&self) -> u32;
    fn open(&mut self) -> Result<(), CanError>;
    fn close(&mut self);
    fn device_info(&self) -> Result<&ZDeviceInfo, CanError>;
    fn is_derive_device(&self) -> bool;
    fn is_online(&self) -> Result<bool, CanError> {
        Err(CanError::OtherError("method not supported".to_owned()))
    }
    fn init_can_chl(&mut self, cfg: Vec<CanChlCfg>) -> Result<(), CanError>;
    fn reset_can_chl(&mut self, channel: u8) -> Result<(), CanError>;
    // fn resistance_state(&self, dev_idx: u32, channel: u8) -> Result<(), CanError>;
    fn read_can_chl_status(&self, channel: u8) -> Result<ZCanChlStatus, CanError>;
    fn read_can_chl_error(&self, channel: u8) -> Result<ZCanChlError, CanError>;
    fn clear_can_buffer(&self, channel: u8) -> Result<(), CanError>;
    fn get_can_num(&self, channel: u8, can_type: ZCanFrameType) -> Result<u32, CanError>;
    fn receive_can(&self, channel: u8, size: u32, timeout: Option<u32>) -> Result<Vec<CanMessage>, CanError>;
    fn transmit_can(&self, channel: u8, frames: Vec<CanMessage>) -> Result<u32, CanError>;
    fn receive_canfd(&self, channel: u8, size: u32, timeout: Option<u32>) -> Result<Vec<CanMessage>, CanError> {
        Err(CanError::OtherError("method not supported".to_owned()))
    }
    fn transmit_canfd(&self, channel: u8, frames: Vec<CanMessage>) -> Result<u32, CanError> {
        Err(CanError::OtherError("method not supported".to_owned()))
    }
    fn init_lin_chl(&mut self, cfg: Vec<ZLinChlCfg>) -> Result<(), CanError> {
        Err(CanError::OtherError("method not supported".to_owned()))
    }
    fn reset_lin_chl(&mut self, channel: u8) -> Result<(), CanError> {
        Err(CanError::OtherError("method not supported".to_owned()))
    }
    fn clear_lin_buffer(&self, channel: u8) -> Result<(), CanError> {
        Err(CanError::OtherError("method not supported".to_owned()))
    }
    fn get_lin_num(&self, channel: u8) -> Result<u32, CanError> {
        Err(CanError::OtherError("method not supported".to_owned()))
    }
    fn receive_lin(&self, channel: u8, size: u32, timeout: Option<u32>) -> Result<Vec<ZLinFrame>, CanError> {
        Err(CanError::OtherError("method not supported".to_owned()))
    }
    fn transmit_lin(&self, channel: u8, frames: Vec<ZLinFrame>) -> Result<u32, CanError> {
        Err(CanError::OtherError("method not supported".to_owned()))
    }
    fn set_lin_subscribe(&self, channel: u8, cfg: Vec<ZLinSubscribe>) -> Result<(), CanError> {
        Err(CanError::OtherError("method not supported".to_owned()))
    }
    fn set_lin_publish(&self, channel: u8, cfg: Vec<ZLinPublish>) -> Result<(), CanError> {
        Err(CanError::OtherError("method not supported".to_owned()))
    }
    fn set_lin_publish_ext(&self, channel: u8, cfg: Vec<ZLinPublishEx>) -> Result<(), CanError> {
        Err(CanError::OtherError("method not supported".to_owned()))
    }
    fn wakeup_lin(&self, channel: u8) -> Result<(), CanError> {
        Err(CanError::OtherError("method not supported".to_owned()))
    }
    #[deprecated(since = "0.1.0", note = "This method is deprecated!")]
    fn set_lin_slave_msg(&self, channel: u8, msg: Vec<ZLinFrame>) -> Result<(), CanError> {
        Err(CanError::OtherError("method not supported".to_owned()))
    }
    #[deprecated(since = "0.1.0", note = "This method is deprecated!")]
    fn clear_lin_slave_msg(&self, channel: u8, pids: Vec<u8>) -> Result<(), CanError> {
        Err(CanError::OtherError("method not supported".to_owned()))
    }
    fn set_server(&self, server: ZCloudServerInfo) -> Result<(), CanError> {
        Err(CanError::OtherError("method not supported".to_owned()))
    }
    fn connect_server(&self, username: &str, password: &str) -> Result<(), CanError> {
        Err(CanError::OtherError("method not supported".to_owned()))
    }
    fn is_connected_server(&self) -> Result<bool, CanError> {
        Err(CanError::OtherError("method not supported".to_owned()))
    }
    fn disconnect_server(&self) -> Result<(), CanError> {
        Err(CanError::OtherError("method not supported".to_owned()))
    }
    fn get_userdata(&self, update: Option<i32>) -> Result<ZCloudUserData, CanError> {
        Err(CanError::OtherError("method not supported".to_owned()))
    }
    fn receive_gps(&self, size: u32, timeout: Option<u32>) -> Result<Vec<ZCloudGpsFrame>, CanError> {
        Err(CanError::OtherError("method not supported".to_owned()))
    }
    fn timestamp(&self, channel: u8) -> Result<u64, CanError>;
    fn device_handler<C, T>(&self, callback: C) -> Result<T, CanError>
        where
            C: FnOnce(&Handler) -> Result<T, CanError>;
    #[inline(always)]
    fn can_handler<C, T>(&self, channel: u8, callback: C) -> Result<T, CanError>
        where
            C: FnOnce(&ZChannelContext) -> Result<T, CanError> {
        self.device_handler(|hdl| -> Result<T, CanError> {
            match hdl.find_can(channel) {
                Some(context) => callback(context),
                None => Err(CanError::ChannelNotOpened(channel.to_string())),
            }
        })
    }

    #[inline(always)]
    fn lin_handler<C, T>(&self, channel: u8, callback: C) -> Result<T, CanError>
        where
            C: FnOnce(&ZChannelContext) -> Result<T, CanError> {
        self.device_handler(|hdl| -> Result<T, CanError> {
            match hdl.lin_channels().get(&channel) {
                Some(chl) => callback(chl),
                None => Err(CanError::ChannelNotOpened(channel.to_string())),
            }
        })
    }
}
