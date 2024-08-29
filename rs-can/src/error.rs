#[derive(thiserror::Error, Debug, Copy, Clone, Eq, PartialEq)]
pub enum CanError {
    #[error("RS-CAN - device initialization error")]
    InitializationError,
    #[error("RS-CAN - device operation error")]
    OperationError,
    #[error("RS-CAN - device timeout error")]
    Timeout,
}
