mod utils;

use zlgcan::device::ZCanDeviceType;
use self::utils::can_device2;

#[test]
fn usbcan_4eu() -> anyhow::Result<()> {
    let dev_type = ZCanDeviceType::ZCAN_USBCAN_4E_U;
    can_device2(dev_type, 4, 4, 0, 1, None)?;
    Ok(())
}

#[test]
fn usbcan_8eu() -> anyhow::Result<()> {
    let dev_type = ZCanDeviceType::ZCAN_USBCAN_8E_U;
    can_device2(dev_type, 8, 8, 0, 1, None)?;
    Ok(())
}

