# A SocketCAN driver

[![Latest version](https://img.shields.io/crates/v/socan.svg)](https://crates.io/crates/socan)
[![Documentation](https://docs.rs/bleasy/badge.svg)](https://docs.rs/socan)
![LGPL](https://img.shields.io/badge/license-LGPL-green.svg)
![MIT](https://img.shields.io/badge/license-MIT-yellow.svg)

## Overview
**socan** is a driver for SocketCAN device.

It is a part of rust-can driver.

### Prerequisites
- Rust 1.70 or higher
- Cargo (included with Rust)

### Adding to Your Project

To use **socan** in your Rust project, add it as a dependency in your `Cargo.toml`:

```toml
[dependencies]
socan = { version="lastest-version" }
```

### UDS example
```toml
[dependencies]
docan = { git = "https://github.com/zhuyu4839/docan-rs", branch = "develop" }
iso14229-1 = { git = "https://github.com/zhuyu4839/iso14229-1", branch = "develop" }
rs-can = { git = "https://github.com/zhuyu4839/ecu-proto-rs", branch = "docan", package = "rs-can", features = ["isotp-std2004"]  }
socan = { git = "https://github.com/zhuyu4839/ecu-proto-rs", branch = "docan", package = "socan" }
anyhow = "1"
```

```shell
sudo ip link add dev vcan0 type vcan
sudo ip link set dev vcan0 up
candump vcan0   # show vcan0 message
```

```rust
use docan::{Client as _, DoCanClient, DoCanServer, Server as _};
use iso14229_1::SessionType;
use rs_can::{Frame, isotp::{Address, AddressType, IsoTpAdapter}};
use socan::{CanMessage, SocketCan};

type Client = (IsoTpAdapter<SocketCan, String, CanMessage>, DoCanClient<SocketCan, String, CanMessage>);
type Server = (IsoTpAdapter<SocketCan, String, CanMessage>, DoCanServer<SocketCan, String, CanMessage>);

fn init_client() -> anyhow::Result<Client> {
    let channel = "vcan0";
    let mut device = SocketCan::new();
    device.init_channel(channel, true)?;

    let mut adapter = IsoTpAdapter::new(device);
    let mut client = DoCanClient::new(adapter.clone(), None);

    client.init_channel(channel.into(), Address {
        tx_id: 0x7E0,
        rx_id: 0x7E8,
        fid: 0x7DF,
    })?;

    adapter.start(100);

    Ok((adapter, client))
}

fn init_server() -> anyhow::Result<Server> {
    let channel = "vcan0";
    let mut device = SocketCan::new();
    device.init_channel(channel, true)?;

    let mut adapter = IsoTpAdapter::new(device);
    let server = DoCanServer::new(adapter.clone(), channel.into(), Address {
        tx_id: 0x7E8,
        rx_id: 0x7E0,
        fid: 0x7DF,
    });

    adapter.start(100);

    Ok((adapter, server))
}

#[test]
fn test_uds() -> anyhow::Result<()> {
    let channel = "vcan0";
    let (_, mut client) = init_client()?;
    let (_, mut server) = init_server()?;

    std::thread::spawn(move || {
        if let Err(e) = server.service_forever(100) {
            println!("docan server start error: {}", e);
        }
    });

    let mut message = CanMessage::new(0x123, &[0x01, 0x02, 0x03, 0x04]).unwrap();
    message.set_channel(channel.to_string());
    client.adapter().sender().send(message)?;

    client.session_ctrl(channel.to_string(), SessionType::Default, false, AddressType::Physical)?;
    client.session_ctrl(channel.to_string(), SessionType::Default, true, AddressType::Physical)?;
    client.session_ctrl(channel.to_string(), SessionType::Default, true, AddressType::Functional)?;

    Ok(())
}
```

## Contributing

We're always looking for users who have thoughts on how to make `socan` better, or users with
interesting use cases.

Of course, we're also happy to accept code contributions for outstanding feature requests!