mod utils;

use zlgcan::device::{DeriveInfo, ZCanDeviceType};
use self::utils::{can_device1, can_device2};

#[test]
fn usbcan_official1() -> anyhow::Result<()> {
    let dev_type = ZCanDeviceType::ZCAN_USBCAN1;
    can_device1(dev_type, None)?;
    Ok(())
}

#[test]
fn usbcan_derive1() -> anyhow::Result<()> {
    let dev_type = ZCanDeviceType::ZCAN_USBCAN1;
    let derive_info = DeriveInfo::new(false, 1);
    can_device1(dev_type, Some(derive_info))?;
    Ok(())
}

#[test]
fn usbcan_official2() {
    // TODO has no this device
}

#[test]
fn usbcan_derive2() -> anyhow::Result<()> {
    let dev_type = ZCanDeviceType::ZCAN_USBCAN2;
    let derive_info = DeriveInfo::new(false, 2);

    can_device2(dev_type, 2, 2, 0, 1, Some(derive_info))?;
    Ok(())
}

