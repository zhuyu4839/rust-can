use std::{any::Any, fmt::Display};
use crate::{CanError, Frame, Id};

#[cfg(not(feature = "async"))]
pub type ResultWrapper<R, E> = Result<R, E>;
#[cfg(feature = "async")]
pub type ResultWrapper<R, E> = impl std::future::Future<Output = Result<R, E>>;

pub trait Listener<C, F: Frame>: Any + Send {
    fn as_any(&self) -> &dyn Any;
    /// Callback when frame transmitting.
    fn on_frame_transmitting(&self, channel: C, frame: &F);
    /// Callback when frame transmit success.
    fn on_frame_transmitted(&self, channel: C, id: Id);
    /// Callback when frames received.
    fn on_frame_received(&self, channel: C, frames: &[F]);
}

pub trait CanDriver: Clone {
    type Channel: Display;
    type Frame: Frame<Channel = Self::Channel>;
    #[inline]
    fn is_closed(&self) -> bool {
        self.opened_channels().is_empty()
    }
    /// get all channels that has opened
    fn opened_channels(&self) -> Vec<Self::Channel>;
    /// Transmit a CAN or CAN-FD Frame.
    fn transmit(&self, msg: Self::Frame, timeout: Option<u32>) -> ResultWrapper<(), CanError>;
    /// Receive CAN and CAN-FD Frames.
    fn receive(&self, channel: Self::Channel, timeout: Option<u32>) -> ResultWrapper<Vec<Self::Frame>, CanError>;
    /// Close CAN device.
    fn shutdown(&mut self);
}
