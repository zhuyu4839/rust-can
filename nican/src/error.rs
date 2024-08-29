#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("NI-CAN - initialization error")]
    NicanInitializationError,
    #[error("NI-CAN - operation error")]
    NicanOperationError,
}
