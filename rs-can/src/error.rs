use thiserror::Error;

#[derive(Debug,  Clone, Error)]
pub enum Error {
    /// Error when operation like library loading, device or channel opening and so on.
    #[error("RUST-CAN - initialize error: {0}")]
    InitializeError(String),
    /// Error when function is not implemented.
    #[error("RUST-CAN - not implement error")]
    NotImplementedError,
    /// Error when operation like transmit, receive and so on.
    #[error("RUST-CAN - operation error: {0}")]
    OperationError(String),
    /// Error when others.
    #[error("RUST-CAN - other error: {0}")]
    OtherError(String),
}
