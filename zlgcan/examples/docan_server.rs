// use docan::Server;
// use rs_can::{CanFrame, CanId};
// use zlgcan_rs::can::CanMessage;
//
// mod utils;
//
// fn main() -> anyhow::Result<()> {
//
//     let (mut device, mut server) = utils::init_server()?;
//
//     let msg = CanMessage::new(CanId::from(0x100), &vec![0x01, 0x02, 0x03]).unwrap();
//     server.adapter().sender().send(msg)?;
//
//     server.service_forever(100)?;
//
//     device.stop();
//
//     Ok(())
// }
