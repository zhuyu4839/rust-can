use docan::{Client, DoCanClient};
use iso14229_1::SessionType;
use nican::{CanMessage, NiCan};
use rs_can::isotp::{Address, AddressType, IsoTpAdapter};

const CHANNEL: &'static str = "CAN0";

type DriverClient = (IsoTpAdapter<NiCan, String, CanMessage>, DoCanClient<NiCan, String, CanMessage>);

fn init_device() -> anyhow::Result<DriverClient> {
    let mut device = NiCan::new(None)?;
    device.open(CHANNEL, vec![], 500_000, true)?;

    let mut adapter = IsoTpAdapter::new(device);
    let mut client = DoCanClient::new(adapter.clone(), None);

    client.init_channel(CHANNEL.into(), Address {
        tx_id: 0x7E0,
        rx_id: 0x7E8,
        fid: 0x7DF,
    })?;

    adapter.start(100);

    Ok((adapter, client))
}

fn main() -> anyhow::Result<()> {
    let (mut adapter, mut client) = init_device()?;

    client.update_address(CHANNEL.into(), Address {
        tx_id: 0x7E4,
        rx_id: 0x7EC,
        fid: 0x7DF,
    })?;

    client.session_ctrl(CHANNEL.into(), SessionType::Extended, false, AddressType::Physical)?;

    adapter.stop();

    Ok(())
}

