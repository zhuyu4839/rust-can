#![allow(unused)]

use crate::{CANFD_FRAME_MAX_SIZE, CAN_FRAME_MAX_SIZE};

pub(crate) const P2_MAX: u16 = 50;         // TODO
pub(crate) const P2_STAR_MAX: u16 = 500;   // TODO
pub(crate) const DEFAULT_P2_START_MS: u64 = 5_000;

#[cfg(not(feature = "isotp-can-fd"))]
pub const SINGLE_FRAME_SIZE_2004: usize = CAN_FRAME_MAX_SIZE - 1;
#[cfg(feature = "isotp-can-fd")]
pub const SINGLE_FRAME_SIZE_2004: usize = CANFD_FRAME_MAX_SIZE - 1;
#[cfg(not(feature = "isotp-can-fd"))]
pub const SINGLE_FRAME_SIZE_2016: usize = CAN_FRAME_MAX_SIZE - 2;
#[cfg(feature = "isotp-can-fd")]
pub const SINGLE_FRAME_SIZE_2016: usize = CANFD_FRAME_MAX_SIZE - 2;

#[cfg(not(feature = "isotp-can-fd"))]
pub const FIRST_FRAME_SIZE_2004: usize = CAN_FRAME_MAX_SIZE - 2;
#[cfg(feature = "isotp-can-fd")]
pub const FIRST_FRAME_SIZE_2004: usize = CANFD_FRAME_MAX_SIZE - 2;
#[cfg(not(feature = "isotp-can-fd"))]
pub const FIRST_FRAME_SIZE_2016: usize = CAN_FRAME_MAX_SIZE - 5;
#[cfg(feature = "isotp-can-fd")]
pub const FIRST_FRAME_SIZE_2016: usize = CANFD_FRAME_MAX_SIZE - 5;

#[cfg(not(feature = "isotp-can-fd"))]
pub const CONSECUTIVE_FRAME_SIZE: usize = CAN_FRAME_MAX_SIZE - 1;
#[cfg(feature = "isotp-can-fd")]
pub const CONSECUTIVE_FRAME_SIZE: usize = CANFD_FRAME_MAX_SIZE - 1;
