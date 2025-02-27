use std::fmt::{Display, Formatter};
use libc::{can_frame, canfd_frame, canxl_frame};
use rs_can::{Direct, IdentifierFlags, EFF_MASK, utils::{system_timestamp, can_dlc, data_resize, is_can_fd_len}, Frame, Id, CAN_FRAME_MAX_SIZE};
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
    pub(crate) is_fd: bool,
    pub(crate) direct: Direct,
    pub(crate) bitrate_switch: bool,
    pub(crate) error_state_indicator: bool,
}

impl From<CanAnyFrame> for CanMessage {
    fn from(frame: CanAnyFrame) -> Self {
        let timestamp = system_timestamp();
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
                is_fd: false,
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
                is_fd: false,
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
                is_fd: false,
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
                is_fd: true,
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
        if self.is_fd {
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
        }
        else {
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
        }
    }
}

impl Frame for CanMessage {
    type Channel = String;

    fn new(id: impl Into<Id>, data: &[u8]) -> Option<Self> {
        let length = data.len();

        match is_can_fd_len(length) {
            Ok(is_fd) => {
                let id: Id = id.into();
                Some(Self {
                    timestamp: 0,
                    arbitration_id: id.as_raw(),
                    is_extended_id: id.is_extended(),
                    is_remote_frame: false,
                    is_error_frame: false,
                    channel: Default::default(),
                    length,
                    data: data.to_vec(),
                    is_fd,
                    direct: Default::default(),
                    bitrate_switch: false,
                    error_state_indicator: false,
                })
            },
            Err(_) => None,
        }
    }

    fn new_remote(id: impl Into<Id>, len: usize) -> Option<Self> {
        match is_can_fd_len(len) {
            Ok(is_fd) => {
                let id = id.into();
                let mut data = Vec::new();
                data_resize(&mut data, len);
                Some(Self {
                    timestamp: 0,
                    arbitration_id: id.as_raw(),
                    is_extended_id: id.is_extended(),
                    is_remote_frame: true,
                    is_error_frame: false,
                    channel: Default::default(),
                    length: len,
                    data,
                    is_fd,
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
        self.timestamp = value.unwrap_or_else(system_timestamp);
        self
    }

    #[inline]
    fn id(&self) -> Id {
        Id::from_bits(self.arbitration_id, self.is_extended_id)
    }

    #[inline]
    fn is_can_fd(&self) -> bool {
        self.is_fd
    }

    #[inline]
    fn set_can_fd(&mut self, value: bool) -> &mut Self {
        if !value {
            match self.length {
                9.. => {
                    log::warn!("resize a fd-frame to: {}", CAN_FRAME_MAX_SIZE);
                    self.length = CAN_FRAME_MAX_SIZE;
                },
                _ => {},
            }
        }
        self.is_fd = value;
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
    fn direct(&self) -> Direct {
        self.direct.clone()
    }

    #[inline]
    fn set_direct(&mut self, direct: Direct) -> &mut Self {
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
    fn dlc(&self) -> Option<usize> {
        can_dlc(self.length, self.is_fd)
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
        <dyn Frame<Channel=String> as Display>::fmt(self, f)
    }
}
