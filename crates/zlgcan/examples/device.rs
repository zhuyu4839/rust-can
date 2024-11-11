mod utils;

use iso15765_2::can::{Frame, Id};
use zlgcan::can::CanMessage;
use zlgcan::driver::ZDevice;
use crate::utils::{init_device, CHANNEL};

fn main() -> anyhow::Result<()> {
    let device = init_device()?;

    let data = vec![0x02, 0x10, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00];
    let msg = CanMessage::new(Id::from(0x7DF), &data).unwrap();

    device.transmit_can(CHANNEL, vec![msg, ])?;

    let results = device.receive_can(CHANNEL, 1, None)?;
    println!("{:?}", results);
    let results = device.receive_canfd(CHANNEL, 1, None)?;
    println!("{:?}", results);

    Ok(())
}
