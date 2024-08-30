//! **`rs-can`**, A unified CAN driver, just like [python-can](https://github.com/hardbyte/python-can)

mod device;
pub use device::*;

pub mod error;
pub mod utils;
