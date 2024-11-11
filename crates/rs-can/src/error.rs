#[derive(thiserror::Error, Debug,  Clone, Eq, PartialEq)]
pub enum CanError {
    #[error("RUST-CAN - device configure failed")]
    DeviceConfigFailed,
    #[error("RUST-CAN - device open failed")]
    DeviceOpenFailed,
    #[error("RUST-CAN - device not supported")]
    DeviceNotSupported,

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
