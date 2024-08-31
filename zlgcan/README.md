# A Cross-platform ZLG(周立功) CAN driver.

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

### Known defects
 * The timestamp of frame is incorrect.

## Contributing

We're always looking for users who have thoughts on how to make `zlgcan` better, or users with
interesting use cases.  

Of course, we're also happy to accept code contributions for outstanding feature requests!
