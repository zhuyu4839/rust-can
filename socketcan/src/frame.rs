use std::fmt::{Display, Formatter};
use libc::{can_frame, canfd_frame, canxl_frame};
use rs_can::{CanDirect, IdentifierFlags, EFF_MASK, can_utils, CanFrame, CanId, MAX_FRAME_SIZE, CanType, MAX_FD_FRAME_SIZE, MAX_XL_FRAME_SIZE};
use crate::{socket, FD_FRAME_SIZE, FRAME_SIZE, XL_FRAME_SIZE};

pub enum CanAnyFrame {
    Normal(can_frame),
    Remote(can_frame),
    Error(can_frame),
    Fd(canfd_frame),
    Xl(canxl_frame)
}

impl CanAnyFrame {
    pub fn size(&self) -> usize {
        match self {
            CanAnyFrame::Normal(_) => FRAME_SIZE,
            CanAnyFrame::Remote(_) => FRAME_SIZE,
            CanAnyFrame::Error(_) => FRAME_SIZE,
            CanAnyFrame::Fd(_) => FD_FRAME_SIZE,
            CanAnyFrame::Xl(_) => XL_FRAME_SIZE,
        }
    }
}

impl From<can_frame> for CanAnyFrame {
    #[inline(always)]
    fn from(frame: can_frame) -> CanAnyFrame {
        let can_id = frame.can_id;
        if can_id & IdentifierFlags::REMOTE.bits() != 0 {
            CanAnyFrame::Remote(frame)
        }
        else if can_id & IdentifierFlags::ERROR.bits() != 0 {
            CanAnyFrame::Error(frame)
        }
        else {
            CanAnyFrame::Normal(frame)
        }
    }
}

impl From<canfd_frame> for CanAnyFrame {
    #[inline(always)]
    fn from(frame: canfd_frame) -> Self {
        CanAnyFrame::Fd(frame)
    }
}

impl From<canxl_frame> for CanAnyFrame {
    fn from(frame: canxl_frame) -> Self {
        CanAnyFrame::Xl(frame)
    }
}

#[derive(Debug, Clone)]
pub struct CanMessage {
    pub(crate) timestamp: u64,
    pub(crate) arbitration_id: u32,
    pub(crate) is_extended_id: bool,
    pub(crate) is_remote_frame: bool,
    pub(crate) is_error_frame: bool,
    pub(crate) channel: String,
    pub(crate) length: usize,
    pub(crate) data: Vec<u8>,
    pub(crate) can_type: CanType,
    pub(crate) direct: CanDirect,
    pub(crate) bitrate_switch: bool,
    pub(crate) error_state_indicator: bool,
}

impl From<CanAnyFrame> for CanMessage {
    fn from(frame: CanAnyFrame) -> Self {
        let timestamp = can_utils::system_timestamp();
        match frame {
            CanAnyFrame::Normal(f) => Self {
                timestamp,
                arbitration_id: f.can_id & EFF_MASK,
                is_extended_id: f.can_id & IdentifierFlags::EXTENDED.bits() != 0,
                is_remote_frame: false,
                is_error_frame: false,
                channel: Default::default(),
                length: f.can_dlc as usize,
                data: f.data[..f.can_dlc as usize].to_vec(),
                can_type: CanType::Can,
                direct: Default::default(),
                bitrate_switch: false,
                error_state_indicator: false,
            },
            CanAnyFrame::Remote(f) => Self {
                timestamp,
                arbitration_id: f.can_id & EFF_MASK,
                is_extended_id: f.can_id & IdentifierFlags::EXTENDED.bits() != 0,
                is_remote_frame: true,
                is_error_frame: false,
                channel: Default::default(),
                length: f.can_dlc as usize,
                data: f.data[..f.can_dlc as usize].to_vec(),
                can_type: CanType::Can,
                direct: Default::default(),
                bitrate_switch: false,
                error_state_indicator: false,
            },
            CanAnyFrame::Error(f) => Self {
                timestamp,
                arbitration_id: f.can_id & EFF_MASK,
                is_extended_id: f.can_id & IdentifierFlags::EXTENDED.bits() != 0,
                is_remote_frame: false,
                is_error_frame: true,
                channel: Default::default(),
                length: f.can_dlc as usize,
                data: f.data[..f.can_dlc as usize].to_vec(),
                can_type: CanType::Can,
                direct: Default::default(),
                bitrate_switch: false,
                error_state_indicator: false,
            },
            CanAnyFrame::Fd(f) => Self {
                timestamp,
                arbitration_id: f.can_id & EFF_MASK,
                is_extended_id: f.can_id & IdentifierFlags::EXTENDED.bits() != 0,
                is_remote_frame: false,
                is_error_frame: false,
                channel: Default::default(),
                length: f.len as usize,
                data: f.data[..f.len as usize].to_vec(),
                can_type: CanType::CanFd,
                direct: Default::default(),
                bitrate_switch: f.flags & 0x01 != 0,
                error_state_indicator: f.flags & 0x02 != 0,
            },
            CanAnyFrame::Xl(_) => todo!(),
        }
    }
}

impl Into<CanAnyFrame> for CanMessage {
    fn into(self) -> CanAnyFrame {
        match self.can_type {
            CanType::Can => {
                let mut frame = socket::can_frame_default();
                let length = self.data.len();
                frame.data[..length].copy_from_slice(&self.data);
                frame.can_dlc = length as u8;
                let mut can_id = self.arbitration_id;
                if self.is_extended_id {
                    can_id |= IdentifierFlags::EXTENDED.bits();
                }

                if self.is_error_frame {
                    can_id |= IdentifierFlags::ERROR.bits();
                    frame.can_id = can_id;
                    return CanAnyFrame::Error(frame);
                }

                if self.is_remote_frame {
                    can_id |= IdentifierFlags::REMOTE.bits();
                    frame.can_id = can_id;
                    return CanAnyFrame::Remote(frame);
                }

                frame.can_id = can_id;
                CanAnyFrame::Normal(frame)
            },
            CanType::CanFd => {
                let mut frame = socket::canfd_frame_default();
                let mut can_id = self.arbitration_id;
                if self.is_extended_id {
                    can_id |= IdentifierFlags::EXTENDED.bits();
                }
                if self.is_remote_frame {
                    can_id |= IdentifierFlags::REMOTE.bits();
                }

                let length = self.data.len();
                frame.can_id = can_id;
                frame.data[..length].copy_from_slice(&self.data);
                frame.len = length as u8;
                if self.bitrate_switch {
                    frame.flags |= 0x01;
                }

                if self.error_state_indicator {
                    frame.flags |= 0x02;
                }

                CanAnyFrame::Fd(frame)
            },
            CanType::CanXl => todo!(),
        }
    }
}

impl CanFrame for CanMessage {
    type Channel = String;

    fn new(id: impl Into<CanId>, data: &[u8]) -> Option<Self> {
        let length = data.len();

        match can_utils::can_type(length) {
            Ok(can_type) => {
                let id: CanId = id.into();
                Some(Self {
                    timestamp: 0,
                    arbitration_id: id.as_raw(),
                    is_extended_id: id.is_extended(),
                    is_remote_frame: false,
                    is_error_frame: false,
                    channel: Default::default(),
                    length,
                    data: data.to_vec(),
                    can_type,
                    direct: Default::default(),
                    bitrate_switch: false,
                    error_state_indicator: false,
                })
            },
            Err(_) => None,
        }
    }

    fn new_remote(id: impl Into<CanId>, len: usize) -> Option<Self> {
        match can_utils::can_type(len) {
            Ok(can_type) => {
                let id = id.into();
                let mut data = Vec::new();
                can_utils::data_resize(&mut data, len);
                Some(Self {
                    timestamp: 0,
                    arbitration_id: id.as_raw(),
                    is_extended_id: id.is_extended(),
                    is_remote_frame: true,
                    is_error_frame: false,
                    channel: Default::default(),
                    length: len,
                    data,
                    can_type,
                    direct: Default::default(),
                    bitrate_switch: false,
                    error_state_indicator: false,
                })
            },
            Err(_) => None,
        }
    }

    #[inline]
    fn timestamp(&self) -> u64 {
        self.timestamp
    }

    #[inline]
    fn set_timestamp(&mut self, value: Option<u64>) -> &mut Self {
        self.timestamp = value.unwrap_or_else(can_utils::system_timestamp);
        self
    }

    #[inline]
    fn id(&self) -> CanId {
        CanId::from_bits(self.arbitration_id, Some(self.is_extended_id))
    }

    #[inline]
    fn can_type(&self) -> CanType {
        self.can_type
    }

    fn set_can_type(&mut self, r#type: CanType) -> &mut Self {
        match r#type {
            CanType::Can => if self.length > MAX_FRAME_SIZE {
                log::warn!("resize a frame to: {}", MAX_FRAME_SIZE);
                self.length = MAX_FRAME_SIZE;
            },
            CanType::CanFd => if self.length > MAX_FD_FRAME_SIZE {
                log::warn!("resize a frame to: {}", MAX_FD_FRAME_SIZE);
                self.length = MAX_FD_FRAME_SIZE;
            },
            CanType::CanXl => if self.length > MAX_XL_FRAME_SIZE {
                log::warn!("resize a frame to: {}", MAX_XL_FRAME_SIZE);
                self.length = MAX_XL_FRAME_SIZE;
            },
        }

        self.can_type = r#type;
        self
    }

    #[inline]
    fn is_remote(&self) -> bool {
        self.is_remote_frame
    }

    #[inline]
    fn is_extended(&self) -> bool {
        self.is_extended_id
    }

    #[inline]
    fn direct(&self) -> CanDirect {
        self.direct.clone()
    }

    #[inline]
    fn set_direct(&mut self, direct: CanDirect) -> &mut Self {
        self.direct = direct;
        self
    }

    #[inline]
    fn is_bitrate_switch(&self) -> bool {
        self.bitrate_switch
    }

    #[inline]
    fn set_bitrate_switch(&mut self, value: bool) -> &mut Self {
        self.bitrate_switch = value;
        self
    }

    #[inline]
    fn is_error_frame(&self) -> bool {
        self.is_error_frame
    }

    #[inline]
    fn set_error_frame(&mut self, value: bool) -> &mut Self {
        self.is_error_frame = value;
        self
    }

    #[inline]
    fn is_esi(&self) -> bool {
        self.error_state_indicator
    }

    #[inline]
    fn set_esi(&mut self, value: bool) -> &mut Self {
        self.error_state_indicator = value;
        self
    }

    #[inline]
    fn channel(&self) -> Self::Channel {
        self.channel.clone()
    }

    #[inline]
    fn set_channel(&mut self, value: Self::Channel) -> &mut Self {
        self.channel = value;
        self
    }

    #[inline]
    fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    #[inline]
    fn length(&self) -> usize {
        self.length
    }
}

impl PartialEq for CanMessage {
    fn eq(&self, other: &Self) -> bool {
        if self.length != other.length {
            return false;
        }

        if self.is_remote_frame {
            other.is_remote_frame && (self.arbitration_id == other.arbitration_id)
        }
        else {
            (self.arbitration_id == other.arbitration_id) &&
                (self.is_extended_id == other.is_extended_id) &&
                (self.is_error_frame == other.is_error_frame) &&
                (self.error_state_indicator == other.error_state_indicator) &&
                (self.data == other.data)
        }
    }
}

impl Display for CanMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <dyn CanFrame<Channel=String> as Display>::fmt(self, f)
    }
}
