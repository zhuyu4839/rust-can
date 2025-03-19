use thiserror::Error;

#[derive(Debug,  Clone, Error)]
pub enum Error {
    /// Error when operation like library loading, device or channel opening and so on.
    #[error("RUST-CAN - initialize error: {0}")]
    InitializeError(String),
    /// Error when function is not implemented.
    #[error("RUST-CAN - not implement error")]
    NotImplementedError,
    /// Error when function is not supported.
    #[error("RUST-CAN - not supported error")]
    NotSupportedError,
    /// Error when operation timeout.
    #[error("RUST-CAN - timeout error: {0}")]
    TimeoutError(String),
    /// Error when operation like transmit, receive and so on.
    #[error("RUST-CAN - operation error: {0}")]
    OperationError(String),
    /// Error when others.
    #[error("RUST-CAN - other error: {0}")]
    OtherError(String),
}

impl Error {
    #[inline(always)]
    pub fn device_open_error(msg: &str) -> Self {
        Self::OperationError(format!("{} when device opened", msg))
    }
    #[inline(always)]
    pub fn device_not_opened() -> Self {
        Self::operation_error("device is not opened")
    }
    #[inline(always)]
    pub fn channel_not_opened<T: std::fmt::Display>(channel: T) -> Self {
        Self::OperationError(format!("channel: {} is not opened", channel))
    }
    #[inline(always)]
    pub fn channel_timeout<T: std::fmt::Display>(channel: T) -> Self {
        Self::TimeoutError(format!("at channel: {}", channel))
    }
    #[inline(always)]
    pub fn operation_error(msg: &str) -> Self {
        Self::OperationError(msg.into())
    }
    #[inline(always)]
    pub fn other_error(msg: &str) -> Self {
        Self::OtherError(msg.into())
    }
}
