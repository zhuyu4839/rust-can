use std::fmt::{Display, Formatter};
use rs_can::CanError;

pub(crate) const BITRATE_CFG_FILENAME: &str = "bitrate.cfg.yaml";
pub(crate) const ZCAN_ENV: &str = "zcan.env";
pub(crate) const ZCAN_VAR: &str = "ZCAN_LIBRARY";
pub(crate) const ZCAN_PATH_DEFAULT: &str = "library";
pub(crate) const TIMING0: &str = "timing0";
pub(crate) const TIMING1: &str = "timing1";
pub const TSEG1: &str = "tseg1";     // Time Segment 1
pub const TSEG2: &str = "tseg2";     // Time Segment 2
pub const SJW: &str = "sjw";         // Synchronization Jump Width
pub const SMP: &str = "smp";         // Sampling specifies
pub const BRP: &str = "brp";         // BaudRate Pre-scale

// pub const CAN_EFF_FLAG: u32 = 0x80000000; /* EFF/SFF is set in the MSB */
// pub const CAN_RTR_FLAG: u32 = 0x40000000; /* remote transmission request */
// pub const CAN_ERR_FLAG: u32 = 0x20000000; /* error message frame */
// pub const CAN_SFF_MASK: u32 = 0x000007FF; /* standard frame format (SFF) */
// pub const CAN_ID_FLAG: u32 = 0x1FFFFFFF; /* id */
// pub const CAN_EFF_MASK: u32 = 0x1FFF800;
pub const CANFD_BRS: u8 = 0x01; /* bit rate switch (second bitrate for payload data) */
pub const CANFD_ESI: u8 = 0x02; /* error state indicator of the transmitting node */

// pub const CAN_FRAME_LENGTH: usize = 8;
pub const CANERR_FRAME_LENGTH: usize = 8;
// pub const CANFD_FRAME_LENGTH: usize = 64;
pub(crate) const TIME_FLAG_VALID: u8 = 1;

/// Then CAN frame type used in crate.
#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
pub enum ZCanFrameType {
    CAN = 0,
    CANFD = 1,
    ALL = 2,
}

impl TryFrom<u8> for ZCanFrameType {
    type Error = CanError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ZCanFrameType::CAN),
            1 => Ok(ZCanFrameType::CANFD),
            2 => Ok(ZCanFrameType::ALL),
            _ => Err(CanError::OtherError("parameter not supported".to_owned())),
        }
    }
}

impl Display for ZCanFrameType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CAN => write!(f, "CAN"),
            Self::CANFD => write!(f, "CANFD"),
            Self::ALL => write!(f, "CAN|CANFD"),
        }
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub enum ZCanChlMode {
    #[default]
    Normal = 0,
    ListenOnly = 1,
}

impl TryFrom<u8> for ZCanChlMode {
    type Error = CanError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ZCanChlMode::Normal),
            1 => Ok(ZCanChlMode::ListenOnly),
            _ => Err(CanError::OtherError("parameter not supported".to_owned())),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
pub enum ZCanFdStd {
    CANFD_ISO = 0,
    CANFD_NON_ISO = 1,
}

impl TryFrom<u8> for ZCanFdStd {
    type Error = CanError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ZCanFdStd::CANFD_ISO),
            1 => Ok(ZCanFdStd::CANFD_NON_ISO),
            _ => Err(CanError::OtherError("parameter not supported".to_owned())),
        }
    }
}

#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Debug, Default, Copy, Clone)]
pub enum ZCanChlType {
    #[default]
    CAN = 0,
    CANFD_ISO = 1,
    CANFD_NON_ISO = 2,
}

impl TryFrom<u8> for ZCanChlType {
    type Error = CanError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ZCanChlType::CAN),
            1 => Ok(ZCanChlType::CANFD_ISO),
            2 => Ok(ZCanChlType::CANFD_NON_ISO),
            _ => Err(CanError::OtherError("parameter not supported".to_owned())),
        }
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub enum ZCanFilterType {
    #[default]
    Double = 0,
    Single = 1,
}

impl TryFrom<u8> for ZCanFilterType {
    type Error = CanError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ZCanFilterType::Double),
            1 => Ok(ZCanFilterType::Single),
            _ => Err(CanError::OtherError("parameter not supported".to_owned())),
        }
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub enum ZCanTxMode {
    #[default]
    Normal = 0,             //**< normal transmission */
    Once = 1,               //**< single-shot transmission */
    SelfReception = 2,      //**< self reception */
    SelfReceptionOnce = 3,  //**< single-shot transmission & self reception */
}

impl TryFrom<u8> for ZCanTxMode {
    type Error = CanError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ZCanTxMode::Normal),
            1 => Ok(ZCanTxMode::Once),
            2 => Ok(ZCanTxMode::SelfReception),
            3 => Ok(ZCanTxMode::SelfReceptionOnce),
            _ => Err(CanError::OtherError("parameter not supported".to_owned())),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ZCanHdrInfoField {
    TxMode = 1,
    FrameType = 2,
    IsRemoteFrame = 3,
    IsExtendFrame = 4,
    IsErrorFrame = 5,
    IsBitrateSwitch = 6,
    IsErrorStateIndicator = 7,
}

impl TryFrom<u8> for ZCanHdrInfoField {
    type Error = CanError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(ZCanHdrInfoField::TxMode),
            2 => Ok(ZCanHdrInfoField::FrameType),
            3 => Ok(ZCanHdrInfoField::IsRemoteFrame),
            4 => Ok(ZCanHdrInfoField::IsExtendFrame),
            5 => Ok(ZCanHdrInfoField::IsErrorFrame),
            6 => Ok(ZCanHdrInfoField::IsBitrateSwitch),
            7 => Ok(ZCanHdrInfoField::IsErrorStateIndicator),
            _ => Err(CanError::OtherError("parameter not supported".to_owned())),
        }
    }
}

/// The reference for Linux device
pub enum Reference {
    Filter = 0x14,          // filter setting; @see ZCAN_Filter and ZCanFilterTable
    SkdSend = 0x16,         // timed send setting; @see ZCAN_TTX
    SkdSendStatus = 0x17,   // timed send status; 0-disable, 1-enable
    Resistance = 0x18,      // terminal resistance; 0-disable, 1-enable
    Timeout = 0x44,         // send timeout; range 0~4000ms
}
