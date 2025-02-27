//! `typedef.rs` defined the zlgcan device type and some function supported feature.
use rs_can::CanError;

#[allow(non_camel_case_types, dead_code)]
#[repr(C)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ZCanDeviceType {
    Undefined                          = 0,
    ZCAN_PCI5121                       = 1,
    ZCAN_PCI9810                       = 2,
    ZCAN_USBCAN1                       = 3,
    ZCAN_USBCAN2                       = 4,
    ZCAN_PCI9820                       = 5,
    ZCAN_CAN232                        = 6,
    ZCAN_PCI5110                       = 7,
    ZCAN_CANLITE                       = 8,
    ZCAN_ISA9620                       = 9,
    ZCAN_ISA5420                       = 10,
    ZCAN_PC104CAN                      = 11,
    ZCAN_CANETUDP                      = 12,
    // ZCAN_CANETE                        = 12,
    ZCAN_DNP9810                       = 13,
    ZCAN_PCI9840                       = 14,
    ZCAN_PC104CAN2                     = 15,
    ZCAN_PCI9820I                      = 16,
    ZCAN_CANETTCP                      = 17,
    ZCAN_PCIE_9220                     = 18,
    ZCAN_PCI5010U                      = 19,
    ZCAN_USBCAN_E_U                    = 20,
    ZCAN_USBCAN_2E_U                   = 21,
    ZCAN_PCI5020U                      = 22,
    ZCAN_EG20T_CAN                     = 23,
    ZCAN_PCIE9221                      = 24,
    ZCAN_WIFICAN_TCP                   = 25,
    ZCAN_WIFICAN_UDP                   = 26,
    ZCAN_PCIe9120                      = 27,
    ZCAN_PCIe9110                      = 28,
    ZCAN_PCIe9140                      = 29,
    ZCAN_USBCAN_4E_U                   = 31,
    ZCAN_CANDTU_200UR                  = 32,
    ZCAN_CANDTU_MINI                   = 33,
    ZCAN_USBCAN_8E_U                   = 34,
    ZCAN_CANREPLAY                     = 35,
    ZCAN_CANDTU_NET                    = 36,
    ZCAN_CANDTU_100UR                  = 37,
    ZCAN_PCIE_CANFD_100U               = 38,
    ZCAN_PCIE_CANFD_200U               = 39,
    ZCAN_PCIE_CANFD_400U               = 40,
    ZCAN_USBCANFD_200U                 = 41,
    ZCAN_USBCANFD_100U                 = 42,
    ZCAN_USBCANFD_MINI                 = 43,
    ZCAN_CANFDCOM_100IE                = 44,
    ZCAN_CANSCOPE                      = 45,
    ZCAN_CLOUD                         = 46,
    ZCAN_CANDTU_NET_400                = 47,
    // ZCAN_CANFDNET_TCP                  = 48,
    ZCAN_CANFDNET_200U_TCP             = 48,
    // ZCAN_CANFDNET_UDP                  = 49,
    ZCAN_CANFDNET_200U_UDP             = 49,
    // ZCAN_CANFDWIFI_TCP                 = 50,
    ZCAN_CANFDWIFI_100U_TCP            = 50,
    // ZCAN_CANFDWIFI_UDP                 = 51,
    ZCAN_CANFDWIFI_100U_UDP            = 51,
    ZCAN_CANFDNET_400U_TCP             = 52,
    ZCAN_CANFDNET_400U_UDP             = 53,
    ZCAN_CANFDBLUE_200U                = 54,
    ZCAN_CANFDNET_100U_TCP             = 55,
    ZCAN_CANFDNET_100U_UDP             = 56,
    ZCAN_CANFDNET_800U_TCP             = 57,
    ZCAN_CANFDNET_800U_UDP             = 58,
    ZCAN_USBCANFD_800U                 = 59,
    ZCAN_PCIE_CANFD_100U_EX            = 60,
    ZCAN_PCIE_CANFD_400U_EX            = 61,
    ZCAN_PCIE_CANFD_200U_MINI          = 62,
    ZCAN_PCIE_CANFD_200U_M2            = 63,
    ZCAN_CANFDDTU_400_TCP              = 64,
    ZCAN_CANFDDTU_400_UDP              = 65,
    ZCAN_CANFDWIFI_200U_TCP            = 66,
    ZCAN_CANFDWIFI_200U_UDP            = 67,
    ZCAN_CANFDDTU_800ER_TCP            = 68,
    ZCAN_CANFDDTU_800ER_UDP            = 69,
    ZCAN_CANFDDTU_800EWGR_TCP          = 70,
    ZCAN_CANFDDTU_800EWGR_UDP          = 71,
    ZCAN_CANFDDTU_600EWGR_TCP          = 72,
    ZCAN_CANFDDTU_600EWGR_UDP          = 73,

    ZCAN_OFFLINE_DEVICE                = 98,
    ZCAN_VIRTUAL_DEVICE                = 99,
}

impl ZCanDeviceType {
    /// Check the device can use fd frame
    pub fn canfd_support(&self) -> bool {
        matches!(
            self,
            ZCanDeviceType::ZCAN_CANFDBLUE_200U |
            ZCanDeviceType::ZCAN_CANFDCOM_100IE |
            ZCanDeviceType::ZCAN_CANFDDTU_400_TCP | ZCanDeviceType::ZCAN_CANFDDTU_400_UDP |
            ZCanDeviceType::ZCAN_CANFDDTU_600EWGR_TCP | ZCanDeviceType::ZCAN_CANFDDTU_600EWGR_UDP |
            ZCanDeviceType::ZCAN_CANFDDTU_800ER_TCP | ZCanDeviceType::ZCAN_CANFDDTU_800ER_UDP |
            ZCanDeviceType::ZCAN_CANFDDTU_800EWGR_TCP | ZCanDeviceType::ZCAN_CANFDDTU_800EWGR_UDP |
            ZCanDeviceType::ZCAN_CANFDNET_100U_TCP | ZCanDeviceType::ZCAN_CANFDNET_100U_UDP |
            ZCanDeviceType::ZCAN_CANFDNET_200U_TCP | ZCanDeviceType::ZCAN_CANFDNET_200U_UDP |
            ZCanDeviceType::ZCAN_CANFDNET_400U_TCP | ZCanDeviceType::ZCAN_CANFDNET_400U_UDP |
            ZCanDeviceType::ZCAN_CANFDNET_800U_TCP | ZCanDeviceType::ZCAN_CANFDNET_800U_UDP |
            ZCanDeviceType::ZCAN_CANFDWIFI_100U_TCP | ZCanDeviceType::ZCAN_CANFDWIFI_100U_UDP |
            ZCanDeviceType::ZCAN_CANFDWIFI_200U_TCP | ZCanDeviceType::ZCAN_CANFDWIFI_200U_UDP |
            ZCanDeviceType::ZCAN_PCIE_CANFD_100U | ZCanDeviceType::ZCAN_PCIE_CANFD_100U_EX |
            ZCanDeviceType::ZCAN_PCIE_CANFD_200U | ZCanDeviceType::ZCAN_PCIE_CANFD_200U_MINI | ZCanDeviceType::ZCAN_PCIE_CANFD_200U_M2 |
            ZCanDeviceType::ZCAN_PCIE_CANFD_400U | ZCanDeviceType::ZCAN_PCIE_CANFD_400U_EX |
            ZCanDeviceType::ZCAN_USBCANFD_MINI | ZCanDeviceType::ZCAN_USBCANFD_100U | ZCanDeviceType::ZCAN_USBCANFD_200U | ZCanDeviceType::ZCAN_USBCANFD_800U
        )
    }
    /// Check the device is supported LIN
    pub const fn lin_support(&self) -> bool{
        matches!(
            self,
            ZCanDeviceType::ZCAN_USBCANFD_200U | ZCanDeviceType::ZCAN_USBCANFD_800U
        )
    }

    pub const fn has_resistance(&self) -> bool {
        !matches!{
            self,
            ZCanDeviceType::ZCAN_USBCAN1 | ZCanDeviceType::ZCAN_USBCAN2
        }
    }

    pub const fn cloud_support(&self) -> bool {
        matches!(
            self,
            ZCanDeviceType::ZCAN_USBCANFD_800U
        )
    }

    pub const fn filter_record_support(&self) -> bool {
        matches!(
            self,
            ZCanDeviceType::ZCAN_PCI5010U | ZCanDeviceType::ZCAN_PCI5020U |
            ZCanDeviceType::ZCAN_USBCAN_2E_U | ZCanDeviceType::ZCAN_USBCAN_4E_U
        )
    }

    pub const fn auto_send_support(&self) -> bool {
        matches!(
            self,
            ZCanDeviceType::ZCAN_USBCAN_2E_U | ZCanDeviceType::ZCAN_USBCAN_4E_U | ZCanDeviceType::ZCAN_USBCAN_8E_U
        )
    }
    /// set value then read and check the value if true
    /// TODO
    pub const fn get_value_support(&self) -> bool {
        true
    }
}

impl From<ZCanDeviceType> for u32 {
    fn from(value: ZCanDeviceType) -> Self {
        value as u32
    }
}

impl TryFrom<u32> for ZCanDeviceType {
    /// Attention!!!
    /// This method is unsafe if the value is too large
    type Error = CanError;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let layout = std::alloc::Layout::new::<ZCanDeviceType>();
        unsafe {
            let vk: *mut ZCanDeviceType = std::alloc::alloc(layout).cast();
            if vk.is_null() {
                Err(CanError::OtherError("method not supported".to_owned()))
            }
            else {
                let ptr = vk as *mut u32;
                *ptr = value;

                Ok(*vk)
            }
        }
    }
}

impl std::fmt::Display for ZCanDeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", *self)
    }
}
