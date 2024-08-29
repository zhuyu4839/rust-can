use crate::error::CanError;
use crate::Frame;
use std::fmt::Display;

#[derive(Debug, Copy, Clone)]
pub struct ChannelConfig {
    pub bitrate: u32,
    pub dbitrate: u32,
}

pub trait CanDevice: Send {
    type Channel: Display;
    /// Open a CAN channel.
    #[cfg(not(any(feature = "async")))]
    fn open(&mut self, channel: Self::Channel) -> Result<(), CanError>;
    #[cfg(any(feature = "async"))]
    fn open(
        &mut self,
        channel: Self::Channel,
    ) -> impl std::future::Future<Output = Result<(), CanError>>;
    /// Transmit a CAN or CAN-FD Frame.
    #[cfg(not(any(feature = "async")))]
    fn transmit(
        &self,
        channel: Self::Channel,
        msg: impl Frame,
        timeout: Option<u32>,
    ) -> Result<(), CanError>;
    #[cfg(any(feature = "async"))]
    fn transmit(
        &self,
        channel: Self::Channel,
        msg: impl Frame,
        timeout: Option<u32>,
    ) -> impl std::future::Future<Output = Result<(), CanError>>;
    /// Receive CAN and CAN-FD Frames.
    #[cfg(not(any(feature = "async")))]
    fn receive(
        &self,
        channel: Self::Channel,
        timeout: Option<u32>,
    ) -> Result<Vec<impl Frame>, CanError>;
    #[cfg(any(feature = "async"))]
    fn receive(
        &self,
        channel: Self::Channel,
        timeout: Option<u32>,
    ) -> impl std::future::Future<Output = Result<Vec<impl Frame>, CanError>>;
    /// Reset a CAN channel.
    #[cfg(not(any(feature = "async")))]
    fn reset(&self, channel: Self::Channel) -> Result<(), CanError>;
    #[cfg(any(feature = "async"))]
    fn reset(
        &self,
        channel: Self::Channel,
    ) -> impl std::future::Future<Output = Result<(), CanError>>;
    /// Shutdown CAN device.
    #[cfg(not(any(feature = "async")))]
    fn shutdown(&self, channel: Self::Channel) -> Result<(), CanError>;
    #[cfg(any(feature = "async"))]
    fn shutdown(
        &self,
        channel: Self::Channel,
    ) -> impl std::future::Future<Output = Result<(), CanError>>;
}
