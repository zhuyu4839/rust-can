//! **`rs-can`**, A unified CAN driver, just like [python-can](https://github.com/hardbyte/python-can)

mod device;
pub use device::*;

mod error;
pub use error::*;
pub mod utils;
