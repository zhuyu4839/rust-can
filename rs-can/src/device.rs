use std::{any::Any, collections::HashMap, fmt::Display};
use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use crate::error::Error;
use crate::frame::{Frame, Id};

#[cfg(not(feature = "async"))]
pub type CanResult<R, E> = Result<R, E>;
#[cfg(feature = "async")]
pub type CanResult<R, E> = impl std::future::Future<Output = Result<R, E>>;

pub trait Listener<C, F: Frame>: Send {
    fn as_any(&self) -> &dyn Any;
    /// Callback when frame transmitting.
    fn on_frame_transmitting(&self, channel: C, frame: &F);
    /// Callback when frame transmit success.
    fn on_frame_transmitted(&self, channel: C, id: Id);
    /// Callback when frames received.
    fn on_frame_received(&self, channel: C, frames: &[F]);
}

pub trait Device: Clone + TryFrom<DeviceBuilder, Error = Error> {
    type Channel: Display;
    type Frame: Frame<Channel = Self::Channel>;
    #[inline]
    fn is_closed(&self) -> bool {
        self.opened_channels().is_empty()
    }
    /// get all channels that has opened
    fn opened_channels(&self) -> Vec<Self::Channel>;
    /// Transmit a CAN or CAN-FD Frame.
    fn transmit(&self, msg: Self::Frame, timeout: Option<u32>) -> CanResult<(), Error>;
    /// Receive CAN and CAN-FD Frames.
    fn receive(&self, channel: Self::Channel, timeout: Option<u32>) -> CanResult<Vec<Self::Frame>, Error>;
    /// Close CAN device.
    fn shutdown(&mut self);
}

#[derive(Debug, Default, Deserialize, Serialize, Getters)]
pub struct ChannelConfig {
    #[getter(copy)]
    bitrate: u32,
    #[getter(copy)]
    dbitrate: Option<u32>,
    #[getter(copy)]
    resistance: Option<bool>,
    #[serde(skip)]
    others: HashMap<String, Box<dyn Any>>,
}

impl ChannelConfig {
    pub fn new(bitrate: u32) -> Self {
        Self {
            bitrate,
            ..Default::default()
        }
    }

    pub fn set_data_bitrate(&mut self, bitrate: u32) -> &mut Self {
        self.dbitrate = Some(bitrate);
        self
    }

    pub fn set_resistance(&mut self, resistance: bool) -> &mut Self {
        self.resistance = Some(resistance);
        self
    }

    pub fn add_other(&mut self, name: String, other: Box<dyn Any>) -> &mut Self {
        self.others.insert(name, other);
        self
    }

    pub fn get_other<T: Clone + 'static>(&self, name: &str) -> Result<Option<T>, Error> {
        get_other(&self.others, name)
    }
}

#[derive(Debug, Default, Getters)]
pub struct DeviceBuilder {
    interface: String,
    channel: Option<String>,
    #[getter(rename = "channel_configs")]
    configs: HashMap<String, ChannelConfig>,
    others: HashMap<String, Box<dyn Any>>,
}

impl DeviceBuilder {
    pub fn new(interface: &str) -> Self {
        Self {
            interface: interface.into(),
            ..Default::default()
        }
    }

    pub fn set_channel<S: Into<String>>(&mut self, channel: S) -> &mut Self {
        self.channel = Some(channel.into());
        self
    }

    pub fn add_config<S: Into<String>>(&mut self,channel: S, cfg: ChannelConfig) -> &mut Self {
        self.configs.insert(channel.into(), cfg);
        self
    }

    pub fn add_other(&mut self, name: &str, cfg: Box<dyn Any>) -> &mut Self {
        self.others.insert(name.into(), cfg);
        self
    }

    pub fn get_other<T: Clone + 'static>(&self, name: &str) -> Result<Option<T>, Error> {
        get_other(&self.others, name)
    }

    pub fn build<T: Device>(self) -> Result<T, Error> {
        self.try_into()
    }
}

#[inline(always)]
fn get_other<T: Clone + 'static>(
    others: &HashMap<String, Box<dyn Any>>,
    name: &str
) -> Result<Option<T>, Error> {
    match others.get(name)  {
        Some(v) => Ok(Some(
            v.downcast_ref::<T>()
                .ok_or(Error::other_error("type mismatched"))?
                .clone()
        )),
        None => Ok(None),
    }
}
