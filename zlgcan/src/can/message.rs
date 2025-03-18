use std::fmt::{Display, Formatter};
use rs_can::{CanDirect, CanFrame, CanId, MAX_FRAME_SIZE, utils::{can_dlc, data_resize, is_can_fd_len}};
use crate::can::ZCanTxMode;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct CanMessage {
    pub(crate) timestamp: u64,
    pub(crate) arbitration_id: u32,
    pub(crate) is_extended_id: bool,
    pub(crate) is_remote_frame: bool,
    pub(crate) is_error_frame: bool,
    pub(crate) channel: u8,
    pub(crate) length: usize,
    pub(crate) data: Vec<u8>,
    pub(crate) is_fd: bool,
    pub(crate) direct: CanDirect,
    pub(crate) bitrate_switch: bool,
    pub(crate) error_state_indicator: bool,
    pub(crate) tx_mode: Option<u8>,
}

unsafe impl Send for CanMessage {}
unsafe impl Sync for CanMessage {}

impl CanFrame for CanMessage {
    type Channel = u8;
    #[inline]
    fn new(id: impl Into<CanId>, data: &[u8]) -> Option<Self> {
        let length = data.len();

        match is_can_fd_len(length) {
            Ok(is_fd) => {
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
                    is_fd,
                    direct: Default::default(),
                    bitrate_switch: false,
                    error_state_indicator: false,
                    tx_mode: Default::default(),
                })
            },
            Err(_) => None,
        }
    }

    #[inline]
    fn new_remote(id: impl Into<CanId>, len: usize) -> Option<Self> {
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
                    tx_mode: Default::default(),
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
        self.timestamp = value.unwrap_or_default();
        self
    }

    #[inline]
    fn id(&self) -> CanId {
        CanId::from_bits(self.arbitration_id, Some(self.is_extended_id))
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
                    log::warn!("resize a fd-frame to: {}", MAX_FRAME_SIZE);
                    self.length = MAX_FRAME_SIZE;
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
    fn direct(&self) -> CanDirect {
        self.direct.clone()
    }

    #[inline]
    fn set_direct(&mut self, direct: CanDirect) -> &mut Self where Self: Sized {
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
    pub fn tx_mode(&self) -> u8 {
        self.tx_mode.unwrap_or_else(|| ZCanTxMode::default() as u8)
    }
    #[inline(always)]
    pub fn set_tx_mode(&mut self, tx_mode: u8) -> &mut Self {
        self.tx_mode = if tx_mode > 3 { Some(ZCanTxMode::default() as u8) } else { Some(tx_mode) };
        self
    }
}

impl Display for CanMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <dyn CanFrame<Channel=u8> as Display>::fmt(self, f)
    }
}
