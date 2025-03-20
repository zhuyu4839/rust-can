mod utils;

use rs_can::{CanDevice, CanFrame, CanId};
use zlgcan_rs::{can::CanMessage, driver::ZDevice};
use crate::utils::{init_device, CHANNEL};

fn main() -> anyhow::Result<()> {
    let mut device = init_device()?;

    let data = vec![0x02, 0x10, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00];
    let msg = CanMessage::new(CanId::from(0x7DF), &data).unwrap();

    device.transmit_can(CHANNEL, vec![msg, ])?;

    let results = device.receive_can(CHANNEL, 1, None)?;
    results.into_iter()
        .for_each(|f| println!("{}", f));
    let results = device.receive_canfd(CHANNEL, 1, Some(20))?;
    results.into_iter()
        .for_each(|f| println!("{}", f));

    device.shutdown();

    Ok(())
}
