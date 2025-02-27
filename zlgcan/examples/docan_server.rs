use docan::Server;
use rs_can::{Frame, Id};
use zlgcan::can::CanMessage;

mod utils;

fn main() -> anyhow::Result<()> {

    let (mut device, mut server) = utils::init_server()?;

    let msg = CanMessage::new(Id::from_bits(0x100, false), &vec![0x01, 0x02, 0x03]).unwrap();
    server.adapter().sender().send(msg)?;

    server.service_forever(100)?;

    device.stop();

    Ok(())
}
