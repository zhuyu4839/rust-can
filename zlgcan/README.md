[![Latest version](https://img.shields.io/crates/v/zlgcan.svg)](https://crates.io/crates/zlgcan)
[![Documentation](https://docs.rs/bleasy/badge.svg)](https://docs.rs/zlgcan)
![LGPL](https://img.shields.io/badge/license-LGPL-green.svg)
![MIT](https://img.shields.io/badge/license-MIT-yellow.svg)
![Downloads](https://img.shields.io/crates/d/zlgcan)

## Overview
 **zlgcan** is a cross-platform driver for ZLG(周立功) device. Include windows and linux. 
 
 It is a part of rust-can driver.

 It also can use UDS-protocol directly.

 Please refer to `examples` for usage examples

## Device list
 * USBCAN-I/II
 * USBCANFD-200U
 * USNCANFD-400U(only channel 1 and channel 2 can be used)
 * USBCANFD-800U

### Prerequisites
 - Rust 1.70 or higher
 - Cargo (included with Rust)

### Demo

```rust
use rs_can::{CanError, DeviceBuilder, interfaces::ZLGCAN, ChannelConfig};
use zlgcan_rs::{can::{ZCanChlMode, ZCanChlType}, device::ZCanDeviceType, driver::{ZDevice, ZCanDriver}, CHANNEL_MODE, CHANNEL_TYPE, DEVICE_INDEX, DEVICE_TYPE};

fn main() -> Result<(), CanError> {
    let mut builder = DeviceBuilder::new(ZLGCAN);

    let mut ch1_cfg = ChannelConfig::new(500_000);
    ch1_cfg.add_other(CHANNEL_MODE, Box::new(ZCanChlMode::Normal as u8))
        .add_other(CHANNEL_TYPE, Box::new(ZCanChlType::CAN as u8));

    let mut ch2_cfg = ChannelConfig::new(500_000);
    ch2_cfg.add_other(CHANNEL_MODE, Box::new(ZCanChlMode::Normal as u8))
        .add_other(CHANNEL_TYPE, Box::new(ZCanChlType::CAN as u8));

    builder.add_other(DEVICE_TYPE, Box::new(ZCanDeviceType::ZCAN_USBCANFD_200U as u32))
        .add_other(DEVICE_INDEX, Box::new(0))
        .add_config(0.to_string(), ch1_cfg)
        .add_config(1.to_string(), ch2_cfg);

    let device = builder.build::<ZCanDriver>()?;
    // todo something

    Ok(())
}
```

### Adding to Your Project

To use **zlgcan** in your Rust project, add it as a dependency in your `Cargo.toml`:

```toml
[dependencies]
zlgcan = { version="lastest-version" }
```

### Create library and configuration

 * Create folder and ensure the file of folder like:
    ```shell
    ├── bitrate.cfg.yaml
    ├── linux
    │   └── x86_64
    └── windows
        ├── x86
        └── x86_64
    ```
    and copy all files into correct directory.

    The basic [library](https://github.com/zhuyu4839/rust-can/blob/master/zlgcan/library).
    The [bitrate.cfg.yaml](https://github.com/zhuyu4839/rust-can/blob/master/zlgcan/library/bitrate.cfg.yaml)

* Create `zcan.env` at your project path for special `ZCAN_LIBRARY` path, For example:
    ```shell
    ZCAN_LIBRARY=/path/to/your/created
    ```

### Known defects
 * The timestamp of frame is incorrect.

## Contributing

We're always looking for users who have thoughts on how to make `zlgcan` better, or users with
interesting use cases.  

Of course, we're also happy to accept code contributions for outstanding feature requests!
