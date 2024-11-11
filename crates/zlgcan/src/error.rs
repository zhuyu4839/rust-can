use thiserror::Error;

#[derive(Debug, Error)]
pub enum ZCanError {
    #[error("ZLGCAN - Invalid device type!")]
    InvalidDeviceType,
    #[error("ZLGCAN - Library load failed: {0}!")]
    LibraryLoadFailed(String),
    #[error("ZLGCAN - Device is not supported!")]
    DeviceNotSupported,
    #[error("ZLGCAN - Device is not supported!")]
    MethodNotSupported,
    #[error("ZLGCAN - Device is not opened!")]
    DeviceNotOpened,
    #[error("ZLGCAN - Channel is not opened!")]
    ChannelNotOpened,
    #[error("ZLGCAN - Parameter is not supported!")]
    ParamNotSupported,
    #[error("ZLGCAN - Method: {0} execute failed, code: {1}!")]
    MethodExecuteFailed(String, u32),
    #[error("ZLGCAN - The configuration error: {0}!")]
    ConfigurationError(String),
    #[error("ZLGCAN - Invalid device context!")]
    InvalidDeviceContext,
    #[error("ZLGCAN - Invalid channel context!")]
    InvalidChannelContext,
    #[error("ZLGCAN - {0}")]
    Other(String),
    #[error("ZLGCAN - Error: {0} when convert to CString!")]
    CStringConvertFailed(String),
    #[error("ZLGCAN - Message convert failed!")]
    MessageConvertFailed,
    #[error("ZLGCAN - No message received!")]
    NoMessageReceived,
}
