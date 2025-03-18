#![allow(unused)]

pub(crate) const BITRATE_CFG_FILENAME: &str = "bitrate.cfg.yaml";
pub(crate) const ZCAN_ENV: &str = "zcan.env";
pub(crate) const ZCAN_VAR: &str = "ZCAN_LIBRARY";
pub(crate) const ZCAN_PATH_DEFAULT: &str = "library";
pub(crate) const TIMING0: &str = "timing0";
pub(crate) const TIMING1: &str = "timing1";
pub(crate) const TSEG1: &str = "tseg1"; // Time Segment 1
pub(crate) const TSEG2: &str = "tseg2"; // Time Segment 2
pub(crate) const SJW: &str = "sjw";     // Synchronization Jump Width
pub(crate) const SMP: &str = "smp";     // Sampling specifies
pub(crate) const BRP: &str = "brp";     // BaudRate Pre-scale

pub(crate) const CANFD_BRS: u8 = 0x01;  /* bit rate switch (second bitrate for payload data) */
pub(crate) const CANFD_ESI: u8 = 0x02;  /* error state indicator of the transmitting node */

// pub const CAN_FRAME_LENGTH: usize = 8;
pub(crate) const CANERR_FRAME_LENGTH: usize = 8;
// pub const CANFD_FRAME_LENGTH: usize = 64;
pub(crate) const TIME_FLAG_VALID: u8 = 1;
