use std::{any::Any, fmt::Display};
use crate::error::Error;
use crate::frame::{Frame, Id};

#[cfg(not(feature = "async"))]
pub type CanResult<R, E> = Result<R, E>;
#[cfg(feature = "async")]
pub type CanResult<R, E> = impl std::future::Future<Output = Result<R, E>>;

pub trait Listener<C, F: Frame>: Any + Send {
    fn as_any(&self) -> &dyn Any;
    /// Callback when frame transmitting.
    fn on_frame_transmitting(&self, channel: C, frame: &F);
    /// Callback when frame transmit success.
    fn on_frame_transmitted(&self, channel: C, id: Id);
    /// Callback when frames received.
    fn on_frame_received(&self, channel: C, frames: &[F]);
}

pub trait Device: Clone {
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
