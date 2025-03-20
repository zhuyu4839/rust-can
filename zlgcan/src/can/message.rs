use std::fmt::{Display, Formatter};
use rs_can::{CanDirect, CanFrame, CanId, CanType, MAX_FRAME_SIZE, MAX_FD_FRAME_SIZE, MAX_XL_FRAME_SIZE, can_utils};
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
    pub(crate) can_type: CanType,
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
                    tx_mode: Default::default(),
                })
            },
            Err(_) => None,
        }
    }

    #[inline]
    fn new_remote(id: impl Into<CanId>, len: usize) -> Option<Self> {
        match can_utils::can_type(len) {
            Ok(can_type) => {
                let id = id.into();

                Some(Self {
                    timestamp: 0,
                    arbitration_id: id.as_raw(),
                    is_extended_id: id.is_extended(),
                    is_remote_frame: true,
                    is_error_frame: false,
                    channel: Default::default(),
                    length: len,
                    data: Default::default(),
                    can_type,
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
    fn set_timestamp(&mut self, value: Option<u64>) -> &mut Self {
        self.timestamp = value.unwrap_or_default();
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

    #[inline]
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
        self.channel
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
