use zlgcan_rs::{can::{ZCanChlMode, ZCanChlType}, device::ZCanDeviceType, driver::ZCanDriver, CHANNEL_MODE, CHANNEL_TYPE, DEVICE_INDEX, DEVICE_TYPE};
use rs_can::{CanError, DeviceBuilder, interfaces::ZLGCAN, ChannelConfig};

pub const CHANNEL: u8 = 0;

pub fn init_device() -> Result<ZCanDriver, CanError> {
    let dev_type = ZCanDeviceType::ZCAN_USBCANFD_200U;

    let mut builder = DeviceBuilder::new(ZLGCAN);

    let mut ch1_cfg = ChannelConfig::new(500_000);
    ch1_cfg.add_other(CHANNEL_MODE, Box::new(ZCanChlMode::Normal as u8))
        .add_other(CHANNEL_TYPE, Box::new(ZCanChlType::CAN as u8));

    let mut ch2_cfg = ChannelConfig::new(500_000);
    ch2_cfg.add_other(CHANNEL_MODE, Box::new(ZCanChlMode::Normal as u8))
        .add_other(CHANNEL_TYPE, Box::new(ZCanChlType::CAN as u8));

    builder.add_other(DEVICE_TYPE, Box::new(dev_type as u32))
        .add_other(DEVICE_INDEX, Box::new(0))
        .add_config(0.to_string(), ch1_cfg)
        .add_config(1.to_string(), ch2_cfg);

    let device = builder.build::<ZCanDriver>()?;

    Ok(device)
}
