# A NI-CAN driver

[![Latest version](https://img.shields.io/crates/v/nican.svg)](https://crates.io/crates/nican)
[![Documentation](https://docs.rs/bleasy/badge.svg)](https://docs.rs/nican)
![LGPL](https://img.shields.io/badge/license-LGPL-green.svg)
![MIT](https://img.shields.io/badge/license-MIT-yellow.svg)

## Overview
**nican** is a driver for NI-USNCAN device(Only can use on windows x86 binary).

It is a part of rust-can driver.

It also can use UDS-protocol directly.

Please refer to `examples` for usage examples

## Device list
 - USB-8473

### Prerequisites
- Rust 1.70 or higher
- Cargo (included with Rust)

### Adding to Your Project

To use **nican** in your Rust project, add it as a dependency in your `Cargo.toml`:

```toml
[dependencies]
nican = { version="lastest-version" }
```

## Contributing

We're always looking for users who have thoughts on how to make `nican` better, or users with
interesting use cases.

Of course, we're also happy to accept code contributions for outstanding feature requests!

