mod utils;

use isotp_rs::can::frame::Frame;
use isotp_rs::can::identifier::Id;
use isotp_rs::device::Driver;
use zlgcan::can::CanMessage;
use crate::utils::{init_device, CHANNEL};

fn main() -> anyhow::Result<()> {
    let device = init_device()?;

    let data = vec![0x02, 0x10, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00];
    let mut msg = CanMessage::new(Id::from(0x7DF), &data).unwrap();
    msg.set_channel(CHANNEL);

    device.transmit(msg, None)?;

    let results = device.receive(CHANNEL, None)?;
    println!("{:?}", results);

    Ok(())
}