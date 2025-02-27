#[derive(Debug,  Clone, thiserror::Error)]
pub enum CanError {
    #[error("RUST-CAN - device configuration error {0}")]
    DeviceConfigError(String),
    #[error("RUST-CAN - device open failed")]
    DeviceOpenFailed,
    #[error("RUST-CAN - device is not opened")]
    DeviceNotOpened,
    #[error("RUST-CAN - device not supported")]
    DeviceNotSupported,

    #[error("RUST-CAN - data length: {0} is too large")]
    DataOutOfRange(usize),

    #[error("RUST-CAN - channel: {0} initialize failed")]
    ChannelInitializeError(String),
    #[error("RUST-CAN - channel: {0} not opened")]
    ChannelNotOpened(String),

    #[error("RUST-CAN - operation error: {0}")]
    OperationError(String),
    #[error("RUST-CAN - channel: {0} timeout error")]
    TimeoutError(String),

    #[error("RUST-CAN - frame convert failed, reason: {0}")]
    FrameConvertFailed(String),

    #[error("RUST-CAN - other error: {0}")]
    OtherError(String),
}
