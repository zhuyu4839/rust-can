use docan::client::SyncClient;
use iso14229_1::SessionType;
use iso15765_2::can::{Address, driver::SyncCan};
use nican::{CanMessage, NiCan};

const CHANNEL: &'static str = "CAN0";

type DriverClient = (SyncCan<NiCan, String, CanMessage>, SyncClient<NiCan, String, CanMessage>);

fn init_device() -> anyhow::Result<DriverClient> {
    let mut device = NiCan::new();
    device.open(CHANNEL, vec![], 500_000, true)?;

    let mut driver = SyncCan::new(device);
    let mut client = SyncClient::new(driver.clone());

    client.init_channel(CHANNEL.into(), Address {
        tx_id: 0x7E0,
        rx_id: 0x7E8,
        fid: 0x7DF,
    }, None)?;

    driver.sync_start(100);

    Ok((driver, client))
}

fn main() -> anyhow::Result<()> {
    let (mut driver, mut client) = init_device()?;

    client.update_address(CHANNEL.into(), Address {
        tx_id: 0x7E4,
        rx_id: 0x7EC,
        fid: 0x7DF,
    })?;

    client.session_ctrl(CHANNEL.into(), SessionType::Extended, false, false)?;

    driver.stop();

    Ok(())
}

