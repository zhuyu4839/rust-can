use rs_can::{{CanDirect, CanFrame, CanId}, can_utils, CanType};
use std::fmt::{Display, Formatter};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct CanMessage {
    timestamp: u64,
    arbitration_id: u32,
    is_extended_id: bool,
    is_remote_frame: bool,
    is_error_frame: bool,
    channel: String,
    length: usize,
    data: Vec<u8>,
    direct: CanDirect,
    bitrate_switch: bool,
    error_state_indicator: bool,
}

unsafe impl Send for CanMessage {}
unsafe impl Sync for CanMessage {}

impl CanFrame for CanMessage {
    type Channel = String;
    #[inline]
    fn new(id: impl Into<CanId>, data: &[u8]) -> Option<Self> {
        let length = data.len();

        match length {
            0..=8 => {
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
                    direct: Default::default(),
                    bitrate_switch: false,
                    error_state_indicator: false,
                })
            }
            _ => None,
        }
    }

    #[inline]
    fn new_remote(id: impl Into<CanId>, len: usize) -> Option<Self> {
        match len {
            0..=8 => {
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
                    direct: Default::default(),
                    bitrate_switch: false,
                    error_state_indicator: false,
                })
            }
            _ => None,
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
        CanType::Can
    }

    #[inline]
    fn set_can_type(&mut self, _: bool) -> &mut Self {
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

impl Display for CanMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <dyn CanFrame<Channel = String> as Display>::fmt(self, f)
    }
}
