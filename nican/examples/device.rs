use std::time::Duration;
use iso15765_2::can::{Id, Frame};
use nican::{CanMessage, NiCan};

fn main() -> anyhow::Result<()> {
    let channel = "CAN0";
    let mut driver = NiCan::new();
    driver.open(channel, vec![], 500_000, true)?;

    let data = vec![0x02, 0x10, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00];
    let mut count = 0;
    loop {
        let mut msg = CanMessage::new(Id::from(0x7DF), data.as_slice()).unwrap();
        msg.set_channel(channel.into());
        driver.transmit_can(msg)?;

        std::thread::sleep(Duration::from_millis(5));
        if let Ok(recv) = driver.receive_can(channel.into(), Some(10)) {
            recv.into_iter()
                .for_each(|msg| println!("{}", msg));
        }
        std::thread::sleep(Duration::from_millis(100));

        count += 1;
        if count > 10 {
            break;
        }
    }

    driver.close(channel.into())?;

    Ok(())
}
