use std::sync::Arc;
use dlopen2::symbor::Container;
use rs_can::{CanError, ChannelConfig};
use crate::can::{CanMessage, ZCanChlError, ZCanChlStatus, ZCanFrameType, constant::{ZCAN_VAR, ZCAN_ENV, ZCAN_PATH_DEFAULT}};
use crate::cloud::{ZCloudGpsFrame, ZCloudServerInfo, ZCloudUserData};
use crate::device::{DeriveInfo, Handler, ZCanDeviceType, ZChannelContext, ZDeviceContext, ZDeviceInfo};
use crate::lin::{ZLinChlCfg, ZLinFrame, ZLinPublish, ZLinPublishEx, ZLinSubscribe};
use crate::api::{WinApi, ZCanApi, ZCloudApi, ZDeviceApi, ZLinApi};
use crate::driver::ZDevice;

#[cfg(target_arch = "x86")]
const LIB_PATH: &str = "windows/x86/";
#[cfg(target_arch = "x86_64")]
const LIB_PATH: &str = "windows/x86_64/";

#[derive(Clone)]
pub struct ZCanDriver {
    pub(crate) handler:    Option<Handler>,
    pub(crate) api:        Arc<Container<WinApi<'static>>>,
    pub(crate) dev_type:   ZCanDeviceType,
    pub(crate) dev_idx:    u32,
    pub(crate) derive:     Option<DeriveInfo>,
}

impl ZDevice for ZCanDriver {
    fn new(dev_type: u32, dev_idx: u32, derive: Option<DeriveInfo>) -> Result<Self, CanError> {
        let libpath = match dotenvy::from_filename(ZCAN_ENV) {
            Ok(_) => match std::env::var(ZCAN_VAR) {
                Ok(v) => format!("{}/{}", v, LIB_PATH),
                Err(_) => format!("{}/{}", ZCAN_PATH_DEFAULT, LIB_PATH),
            },
            Err(_) => format!("{}/{}", ZCAN_PATH_DEFAULT, LIB_PATH),
        };
        let libpath = format!("{}zlgcan.dll", libpath);
        let api =  Arc::new(unsafe {
            Container::load(&libpath)
                .map_err(|_| CanError::InitializeError(format!("can't open library: {}", libpath)))
        }?);
        let dev_type = ZCanDeviceType::try_from(dev_type)?;
        Ok(Self { handler: Default::default(), api, dev_type, dev_idx, derive })
    }

    fn device_type(&self) -> ZCanDeviceType {
        self.dev_type
    }

    fn device_index(&self) -> u32 {
        self.dev_idx
    }

    fn open(&mut self) -> Result<(), CanError> {
        let mut context = ZDeviceContext::new(self.dev_type, self.dev_idx, self.derive.is_some());
        self.api.open(&mut context)?;
        let dev_info = match &self.derive {
            Some(v) => ZDeviceInfo::try_from(v)?,
            None => self.api.read_device_info(&context)?,
        };

        self.handler = Some(Handler::new(context, dev_info));
        Ok(())
    }

    fn close(&mut self) {
        if let Some(handler) = &mut self.handler {
            for (idx, hdl) in handler.can_channels() {
                log::info!("ZLGCAN - closing CAN channel: {}", *idx);
                // let hdl = *hdl;
                self.api.reset_can_chl(hdl).unwrap_or_else(|e| log::warn!("{}", e));
            }
            for (idx, hdl) in handler.lin_channels() {
                log::info!("ZLGCAN - closing LIN channel: {}", *idx);
                // let hdl = *hdl;
                self.api.reset_lin_chl(hdl).unwrap_or_else(|e| log::warn!("{}", e));
            }

            self.api.close(handler.device_context()).unwrap_or_else(|e| log::warn!("{}", e));
            self.handler = None
        }
    }

    fn device_info(&self) -> Result<&ZDeviceInfo, CanError> {
        match &self.handler {
            Some(handler) => Ok(&handler.device_info()),
            None => Err(CanError::device_not_opened()),
        }
    }

    fn is_derive_device(&self) -> bool {
        self.derive.is_some()
    }

    fn is_online(&self) -> Result<bool, CanError> {
        self.device_handler(|hdl| -> Result<bool, CanError> {
            self.api.is_online(hdl.device_context())
        })
    }

    fn init_can_chl(&mut self, channel: u8, cfg: &ChannelConfig) -> Result<(), CanError> {
        match &mut self.handler {
            Some(dev_hdl) => {
                let dev_info = dev_hdl.device_info();
                let channels = dev_info.can_channels();
                if channel >= channels {
                    return Err(CanError::OtherError(format!("CAN channel: {} is out of channels!", channel)));
                }

                if let Some(v) = dev_hdl.find_can(channel) {
                    self.api.reset_can_chl(&v).unwrap_or_else(|e| log::warn!("{}", e));
                    dev_hdl.remove_can(channel);
                }

                let mut context =  ZChannelContext::new(dev_hdl.device_context().clone(), channel);
                self.api.init_can_chl(&mut context, &cfg)?;

                dev_hdl.add_can(channel, context);

                Ok(())
            },
            None => Err(CanError::device_not_opened()),
        }
    }

    fn reset_can_chl(&mut self, channel: u8) -> Result<(), CanError> {
        match &mut self.handler {
            Some(dev_hdl) => {
                match dev_hdl.find_can(channel) {
                    Some(v) => {
                        self.api.reset_can_chl(&v)?;
                        dev_hdl.remove_can(channel);
                        Ok(())
                    },
                    None => Err(CanError::channel_not_opened(channel)),
                }
            },
            None => Err(CanError::device_not_opened()),
        }
    }

    fn read_can_chl_status(&self, channel: u8) -> Result<ZCanChlStatus, CanError> {
        self.can_handler(channel, |context| {
            self.api.read_can_chl_status(context)
        })
    }

    fn read_can_chl_error(&self, channel: u8) -> Result<ZCanChlError, CanError> {
        self.can_handler(channel, |context| {
            self.api.read_can_chl_error(context)
        })
    }

    fn clear_can_buffer(&self, channel: u8) -> Result<(), CanError> {
        self.can_handler(channel, |context| {
            self.api.clear_can_buffer(context)
        })
    }

    fn get_can_num(&self, channel: u8, can_type: ZCanFrameType) -> Result<u32, CanError> {
        self.can_handler(channel, |context| {
            self.api.get_can_num(context, can_type)
        })
    }

    fn receive_can(&self, channel: u8, size: u32, timeout: Option<u32>) -> Result<Vec<CanMessage>, CanError> {
        let timeout = timeout.unwrap_or(u32::MAX);
        self.can_handler(channel, |context| {
            self.api.receive_can(context, size, timeout)
        })
    }

    fn transmit_can(&self, channel: u8, frames: Vec<CanMessage>) -> Result<u32, CanError> {
        self.can_handler(channel, |context| {
            self.api.transmit_can(context, frames)
        })
    }

    fn receive_canfd(&self, channel: u8, size: u32, timeout: Option<u32>) -> Result<Vec<CanMessage>, CanError> {
        let timeout = timeout.unwrap_or(u32::MAX);
        self.can_handler(channel, |context| {
            self.api.receive_canfd(context, size, timeout)
        })
    }

    fn transmit_canfd(&self, channel: u8, frames: Vec<CanMessage>) -> Result<u32, CanError> {
        self.can_handler(channel, |context| {
            self.api.transmit_canfd(context, frames)
        })
    }

    fn init_lin_chl(&mut self, channel: u8, cfg: ZLinChlCfg) -> Result<(), CanError> {
        super::lin_support(self.dev_type)?;
        match &mut self.handler {
            Some(dev_hdl) => {
                let channels = 2;   //dev_info.lin_channels();  // TODO
                if channel >= channels {
                    return Err(CanError::OtherError(format!("LIN channel: {} is out of channels!", channel)));
                }

                if let Some(v) = dev_hdl.find_lin(channel) {
                    self.api.reset_lin_chl(&v).unwrap_or_else(|e| log::warn!("{}", e));
                    dev_hdl.remove_lin(channel);
                }

                let mut context = ZChannelContext::new(dev_hdl.device_context().clone(), channel);
                self.api.init_lin_chl(&mut context, &cfg)?;
                dev_hdl.add_lin(channel, context);

                Ok(())
            },
            None => Err(CanError::device_not_opened()),
        }
    }

    fn reset_lin_chl(&mut self, channel: u8) -> Result<(), CanError> {
        super::lin_support(self.dev_type)?;
        match &mut self.handler {
            Some(dev_hdl) => {
                match dev_hdl.find_lin(channel) {
                    Some(v) => {
                        self.api.reset_lin_chl(&v)?;
                        dev_hdl.remove_lin(channel);
                        Ok(())
                    },
                    None => Err(CanError::channel_not_opened(channel)),
                }
            },
            None => Err(CanError::device_not_opened()),
        }
    }

    fn get_lin_num(&self, channel: u8) -> Result<u32, CanError> {
        super::lin_support(self.dev_type)?;
        self.lin_handler(channel, |context| {
            self.api.get_lin_num(context)
        })
    }

    fn receive_lin(&self, channel: u8, size: u32, timeout: Option<u32>) -> Result<Vec<ZLinFrame>, CanError> {
        super::lin_support(self.dev_type)?;
        let timeout = timeout.unwrap_or(u32::MAX);
        self.lin_handler(channel, |context| {
            self.api.receive_lin(context, size, timeout)
        })
    }

    fn transmit_lin(&self, channel: u8, frames: Vec<ZLinFrame>) -> Result<u32, CanError> {
        super::lin_support(self.dev_type)?;
        self.lin_handler(channel, |context| {
            self.api.transmit_lin(context, frames)
        })
    }

    fn set_lin_subscribe(&self, channel: u8, cfg: Vec<ZLinSubscribe>) -> Result<(), CanError> {
        super::lin_support(self.dev_type)?;
        self.lin_handler(channel, |context| {
            self.api.set_lin_subscribe(context, cfg)
        })
    }

    fn set_lin_publish(&self, channel: u8, cfg: Vec<ZLinPublish>) -> Result<(), CanError> {
        super::lin_support(self.dev_type)?;
        self.lin_handler(channel, |context| {
            self.api.set_lin_publish(context, cfg)
        })
    }

    fn set_lin_publish_ext(&self, channel: u8, cfg: Vec<ZLinPublishEx>) -> Result<(), CanError> {
        super::lin_support(self.dev_type)?;
        self.lin_handler(channel, |context| {
            self.api.set_lin_publish_ex(context, cfg)
        })
    }

    fn wakeup_lin(&self, channel: u8) -> Result<(), CanError> {
        super::lin_support(self.dev_type)?;
        self.lin_handler(channel, |context| {
            self.api.wakeup_lin(context)
        })
    }

    #[allow(deprecated)]
    fn set_lin_slave_msg(&self, channel: u8, msg: Vec<ZLinFrame>) -> Result<(), CanError> {
        super::lin_support(self.dev_type)?;
        self.lin_handler(channel, |context| {
            self.api.set_lin_slave_msg(context, msg)
        })
    }

    #[allow(deprecated)]
    fn clear_lin_slave_msg(&self, channel: u8, pids: Vec<u8>) -> Result<(), CanError> {
        super::lin_support(self.dev_type)?;
        self.lin_handler(channel, |context| {
            self.api.clear_lin_slave_msg(context, pids)
        })
    }

    fn set_server(&self, server: ZCloudServerInfo) -> Result<(), CanError> {
        super::cloud_support(self.dev_type)?;
        self.api.set_server(server)
    }

    fn connect_server(&self, username: &str, password: &str) -> Result<(), CanError> {
        super::cloud_support(self.dev_type)?;
        self.api.connect_server(username, password)
    }

    fn is_connected_server(&self) -> Result<bool, CanError> {
        super::cloud_support(self.dev_type)?;
        self.api.is_connected_server()
    }

    fn disconnect_server(&self) -> Result<(), CanError> {
        super::cloud_support(self.dev_type)?;
        self.api.disconnect_server()
    }

    fn get_userdata(&self, update: Option<i32>) -> Result<ZCloudUserData, CanError> {
        super::cloud_support(self.dev_type)?;
        self.api.get_userdata(update.unwrap_or(0))
    }

    fn receive_gps(&self, size: u32, timeout: Option<u32>) -> Result<Vec<ZCloudGpsFrame>, CanError> {
        super::cloud_support(self.dev_type)?;

        let timeout = timeout.unwrap_or(u32::MAX);
        self.device_handler(|hdl| {
            self.api.receive_gps(hdl.device_context(), size, timeout)
        })
    }

    #[inline]
    fn timestamp(&self, channel: u8) -> Result<u64, CanError> {
        self.can_handler(channel, |context| Ok(context.timestamp()))
    }

    fn device_handler<C, T>(&self, callback: C) -> Result<T, CanError>
        where
            C: FnOnce(&Handler) -> Result<T, CanError> {
        match &self.handler {
            Some(v) => callback(v),
            None => Err(CanError::device_not_opened()),
        }
    }
}
