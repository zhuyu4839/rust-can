# A Cross-platform ZLG(周立功) CAN driver.

[![Latest version](https://img.shields.io/crates/v/zlgcan.svg)](https://crates.io/crates/zlgcan)
[![Documentation](https://docs.rs/bleasy/badge.svg)](https://docs.rs/zlgcan)
![LGPL](https://img.shields.io/badge/license-LGPL-green.svg)
![MIT](https://img.shields.io/badge/license-MIT-yellow.svg)

## Overview
 **zlgcan** is a cross-platform driver for ZLG(周立功) device. Including windows and linux. 
 
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

    The basic [library](https://github.com/zhuyu4839/zlgcan-driver-rs/tree/master/zlgcan-driver/library).
    The [bitrate.cfg.yaml](https://github.com/zhuyu4839/zlgcan-driver-rs/blob/master/zlgcan-driver/bitrate.cfg.yaml)

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
