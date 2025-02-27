use std::fmt::{Display, Formatter};
use rs_can::{Direct, Frame, Id, CAN_FRAME_MAX_SIZE, utils::{can_dlc, data_resize, system_timestamp, is_can_fd_len}};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct CanMessage {
    timestamp: u64,
    arbitration_id: u32,
    is_extended_id: bool,
    is_remote_frame: bool,
    is_error_frame: bool,
    channel: u8,
    length: usize,
    data: Vec<u8>,
    is_fd: bool,
    direct: Direct,
    bitrate_switch: bool,
    error_state_indicator: bool,
    tx_mode: u8,
}

unsafe impl Send for CanMessage {}
unsafe impl Sync for CanMessage {}

impl Frame for CanMessage {
    type Channel = u8;
    #[inline]
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
                    tx_mode: 0,
                })
            },
            Err(_) => None,
        }
    }

    #[inline]
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
                    tx_mode: 0,
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
    fn set_timestamp(&mut self, value: Option<u64>) -> &mut Self where Self: Sized {
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
    fn set_can_fd(&mut self, value: bool) -> &mut Self where Self: Sized {
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
    fn set_direct(&mut self, direct: Direct) -> &mut Self where Self: Sized {
        self.direct = direct;
        self
    }

    #[inline]
    fn is_bitrate_switch(&self) -> bool {
        self.bitrate_switch
    }

    #[inline]
    fn set_bitrate_switch(&mut self, value: bool) -> &mut Self where Self: Sized {
        self.bitrate_switch = value;
        self
    }

    #[inline]
    fn is_error_frame(&self) -> bool {
        self.is_error_frame
    }

    #[inline]
    fn set_error_frame(&mut self, value: bool) -> &mut Self where Self: Sized {
        self.is_error_frame = value;
        self
    }

    #[inline]
    fn is_esi(&self) -> bool {
        self.error_state_indicator
    }

    #[inline]
    fn set_esi(&mut self, value: bool) -> &mut Self where Self: Sized {
        self.error_state_indicator = value;
        self
    }

    #[inline]
    fn channel(&self) -> Self::Channel {
        self.channel
    }

    #[inline]
    fn set_channel(&mut self, value: Self::Channel) -> &mut Self where Self: Sized {
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

impl CanMessage {
    #[inline(always)]
    pub const fn tx_mode(&self) -> u8 { self.tx_mode }
    #[inline(always)]
    pub fn set_tx_mode(&mut self, tx_mode: u8) -> &mut Self {
        self.tx_mode = if tx_mode > 3 { Default::default() } else { tx_mode };
        self
    }
}

impl Display for CanMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <dyn Frame<Channel=u8> as Display>::fmt(self, f)
    }
}
