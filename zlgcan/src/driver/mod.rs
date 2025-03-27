use rs_can::{interfaces, CanDevice, CanError, CanFrame, CanResult, CanType, DeviceBuilder};
use crate::can::{CanChlCfg, CanChlCfgFactory, CanMessage, ZCanChlError, ZCanChlStatus, ZCanFrameType};
use crate::cloud::{ZCloudGpsFrame, ZCloudServerInfo, ZCloudUserData};
use crate::constants;
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

impl CanDevice for ZCanDriver {
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

    fn transmit(&self, msg: Self::Frame, _: Option<u32>) -> CanResult<(), CanError> {
        let channel = msg.channel();
        let _ = match msg.can_type() {
            CanType::Can => self.transmit_can(channel, vec![msg, ]),
            CanType::CanFd => self.transmit_canfd(channel, vec![msg, ]),
            CanType::CanXl => Err(CanError::NotSupportedError),
        }?;

        Ok(())
    }

    fn receive(&self, channel: Self::Channel, timeout: Option<u32>) -> CanResult<Vec<Self::Frame>, CanError> {
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

impl TryFrom<DeviceBuilder> for ZCanDriver {
    type Error = CanError;

    fn try_from(builder: DeviceBuilder) -> Result<Self, Self::Error> {
        if builder.interface() != interfaces::ZLGCAN {
            return Err(CanError::interface_not_matched(builder.interface()));
        }

        let dev_type = builder.get_other::<u32>(constants::DEVICE_TYPE)?
            .ok_or(CanError::other_error("`device_type` not found`"))?;
        let dev_idx = builder.get_other::<u32>(constants::DEVICE_INDEX)?
            .ok_or(CanError::other_error("`device_index` not found`"))?;
        let derive = builder.get_other::<DeriveInfo>(constants::DERIVE_INFO)?;

        let mut device = Self::new(dev_type, dev_idx, derive)?;
        device.open()?;

        let factory = CanChlCfgFactory::new()?;
        builder.channel_configs()
            .iter()
            .try_for_each(|(chl, cfg)| {
                let chl = chl.parse::<u8>()
                    .map_err(|_| CanError::other_error("`chl` not a number"))?;
                device.init_can_chl(chl, factory.from_channel_cfg(dev_type, cfg)?)
            })?;

        Ok(device)
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
        Err(CanError::NotSupportedError)
    }
    fn init_can_chl(&mut self, channel: u8, cfg: CanChlCfg) -> Result<(), CanError>;
    fn reset_can_chl(&mut self, channel: u8) -> Result<(), CanError>;
    // fn resistance_state(&self, dev_idx: u32, channel: u8) -> Result<(), CanError>;
    fn read_can_chl_status(&self, channel: u8) -> Result<ZCanChlStatus, CanError>;
    fn read_can_chl_error(&self, channel: u8) -> Result<ZCanChlError, CanError>;
    fn clear_can_buffer(&self, channel: u8) -> Result<(), CanError>;
    fn get_can_num(&self, channel: u8, can_type: ZCanFrameType) -> Result<u32, CanError>;
    fn receive_can(&self, channel: u8, size: u32, timeout: Option<u32>) -> Result<Vec<CanMessage>, CanError>;
    fn transmit_can(&self, channel: u8, frames: Vec<CanMessage>) -> Result<u32, CanError>;
    fn receive_canfd(&self, channel: u8, size: u32, timeout: Option<u32>) -> Result<Vec<CanMessage>, CanError> {
        Err(CanError::NotSupportedError)
    }
    fn transmit_canfd(&self, channel: u8, frames: Vec<CanMessage>) -> Result<u32, CanError> {
        Err(CanError::NotSupportedError)
    }
    fn init_lin_chl(&mut self, channel: u8, cfg: ZLinChlCfg) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
    fn reset_lin_chl(&mut self, channel: u8) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
    fn clear_lin_buffer(&self, channel: u8) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
    fn get_lin_num(&self, channel: u8) -> Result<u32, CanError> {
        Err(CanError::NotSupportedError)
    }
    fn receive_lin(&self, channel: u8, size: u32, timeout: Option<u32>) -> Result<Vec<ZLinFrame>, CanError> {
        Err(CanError::NotSupportedError)
    }
    fn transmit_lin(&self, channel: u8, frames: Vec<ZLinFrame>) -> Result<u32, CanError> {
        Err(CanError::NotSupportedError)
    }
    fn set_lin_subscribe(&self, channel: u8, cfg: Vec<ZLinSubscribe>) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
    fn set_lin_publish(&self, channel: u8, cfg: Vec<ZLinPublish>) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
    fn set_lin_publish_ext(&self, channel: u8, cfg: Vec<ZLinPublishEx>) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
    fn wakeup_lin(&self, channel: u8) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
    #[deprecated(since = "0.1.0", note = "This method is deprecated!")]
    fn set_lin_slave_msg(&self, channel: u8, msg: Vec<ZLinFrame>) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
    #[deprecated(since = "0.1.0", note = "This method is deprecated!")]
    fn clear_lin_slave_msg(&self, channel: u8, pids: Vec<u8>) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
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
    fn get_userdata(&self, update: Option<i32>) -> Result<ZCloudUserData, CanError> {
        Err(CanError::NotSupportedError)
    }
    fn receive_gps(&self, size: u32, timeout: Option<u32>) -> Result<Vec<ZCloudGpsFrame>, CanError> {
        Err(CanError::NotSupportedError)
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
                None => Err(CanError::channel_not_opened(channel)),
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
                None => Err(CanError::channel_not_opened(channel)),
            }
        })
    }
}

/// device is supported LIN
pub(crate) fn lin_support(dev_type: ZCanDeviceType) -> Result<(), CanError> {
    if !dev_type.lin_support() {
        return Err(CanError::NotSupportedError);
    }
    Ok(())
}


/// device is supported CLOUD
#[allow(dead_code)]
pub(crate) fn cloud_support(dev_type: ZCanDeviceType) -> Result<(), CanError> {
    if !dev_type.cloud_support() {
        return Err(CanError::NotSupportedError);
    }
    Ok(())
}
