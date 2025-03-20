mod utils;

use zlgcan_rs::device::ZCanDeviceType;
use self::utils::canfd_device2;

#[test]
fn usbcanfd_200u() -> anyhow::Result<()> {
    let dev_type = ZCanDeviceType::ZCAN_USBCANFD_200U;
    canfd_device2(dev_type, 2, 2, 0, 1)?;
    Ok(())
}

/// `Attention:`
/// The USBCANFD-400U only supported channel0 and channel1 on Linux
#[test]
fn usbcanfd_400u() -> anyhow::Result<()> {
    // TODO USBCANFD-400U channel 3-4 is not supported
    let dev_type = ZCanDeviceType::ZCAN_USBCANFD_200U;
    canfd_device2(dev_type, 4, 2, 0, 1)?;
    Ok(())
}
