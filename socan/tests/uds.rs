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
