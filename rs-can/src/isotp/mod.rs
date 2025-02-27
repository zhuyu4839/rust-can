mod constant;
pub use constant::*;
mod device;
pub use device::*;
mod standard;

use iso15765_2::{FlowControlContext, FlowControlState, FrameType, Iso15765Error, ISO_TP_DEFAULT_BLOCK_SIZE, ISO_TP_DEFAULT_ST_MIN};

use crate::{CAN_FRAME_MAX_SIZE, DEFAULT_PADDING};

/// ISO-TP address
///
/// * `tx_id`: transmit identifier.
/// * `rx_id`: receive identifier.
/// * `fid`: functional address identifier.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Address {
    pub tx_id: u32,
    pub rx_id: u32,
    pub fid: u32,
}

/// ISO-TP address type.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub enum AddressType {
    #[default]
    Physical,
    Functional,
}

/// p2 context
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct P2Context {
    p2: u16,
    p2_star: u16,
}

impl P2Context {
    #[inline]
    pub fn new(mut p2: u16, mut p2_star: u16) -> Self {
        if p2 > P2_MAX {
            p2 = P2_MAX;
        }

        if p2_star > P2_STAR_MAX {
            p2_star = P2_STAR_MAX;
        }

        Self { p2, p2_star }
    }

    pub fn update(&mut self, mut p2: u16, mut p2_star: u16) {
        if p2 > P2_MAX {
            p2 = P2_MAX;
        }

        if p2_star > P2_STAR_MAX {
            p2_star = P2_STAR_MAX;
        }

        self.p2 = p2;
        self.p2_star = p2_star;
    }

    #[inline]
    pub fn p2_ms(&self) -> u64 {
        self.p2 as u64
    }

    #[inline]
    pub fn p2_star_ms(&self) -> u64 {
        (self.p2_star as u64) * 10
    }
}

impl Default for P2Context {
    #[inline]
    fn default() -> Self {
        Self::new(P2_MAX, P2_STAR_MAX)
    }
}

/// ISO-TP frame define.
#[derive(Debug, Clone)]
pub enum IsoTpFrame {
    /// The ISO-TP single frame.
    SingleFrame { data: Vec<u8> },
    /// The ISO-TP first frame.
    FirstFrame { length: u32, data: Vec<u8> },
    /// The ISO-TP consecutive frame.
    ConsecutiveFrame { sequence: u8, data: Vec<u8> },
    /// The ISO-TP flow control frame.
    FlowControlFrame(FlowControlContext)
}

impl From<&IsoTpFrame> for FrameType {
    fn from(value: &IsoTpFrame) -> Self {
        match value {
            IsoTpFrame::SingleFrame { .. } => Self::Single,
            IsoTpFrame::FirstFrame { .. } => Self::First,
            IsoTpFrame::ConsecutiveFrame { .. } => Self::Consecutive,
            IsoTpFrame::FlowControlFrame(..) => Self::FlowControl,
        }
    }
}

unsafe impl Send for IsoTpFrame {}

impl IsoTpFrame {
    pub fn decode<T: AsRef<[u8]>>(data: T) -> Result<Self, Iso15765Error> {
        let data = data.as_ref();
        let length = data.len();
        match length {
            0 => Err(Iso15765Error::EmptyPdu),
            1..=2 => Err(Iso15765Error::InvalidPdu(data.to_vec())),
            3.. => {
                let byte0 = data[0];
                match FrameType::try_from(byte0)? {
                    FrameType::Single => {   // Single frame
                        standard::decode_single(data, byte0, length)
                    },
                    FrameType::First => {   // First frame
                        standard::decode_first(data, byte0, length)
                    },
                    FrameType::Consecutive => {
                        let sequence = byte0 & 0x0F;
                        Ok(Self::ConsecutiveFrame { sequence, data: Vec::from(&data[1..]) })
                    },
                    FrameType::FlowControl => {
                        // let suppress_positive = (data1 & 0x80) == 0x80;
                        let state = FlowControlState::try_from(byte0 & 0x0F)?;
                        let fc = FlowControlContext::new(state, data[1], data[2])?;
                        Ok(Self::FlowControlFrame(fc))
                    },
                }
            }
            // v => Err(IsoTpError::LengthOutOfRange(v)),
        }
    }

    pub fn encode(self, padding: Option<u8>) -> Vec<u8> {
        match self {
            Self::SingleFrame { data } => {
                standard::encode_single(data, padding)
            },
            Self::FirstFrame { length, data } => {
                standard::encode_first(length, data)
            },
            Self::ConsecutiveFrame { sequence, mut data } => {
                let mut result = vec![FrameType::Consecutive as u8 | sequence];
                result.append(&mut data);
                result.resize(CAN_FRAME_MAX_SIZE, padding.unwrap_or(DEFAULT_PADDING));
                result
            },
            Self::FlowControlFrame(context) => {
                let byte0_h: u8 = FrameType::FlowControl.into();
                let byte0_l: u8 = context.state().into();
                let mut result = vec![
                    byte0_h | byte0_l,
                    context.block_size(),
                    context.st_min(),
                ];
                result.resize(CAN_FRAME_MAX_SIZE, padding.unwrap_or(DEFAULT_PADDING));
                result
            },
        }
    }

    #[inline]
    pub fn from_data<T: AsRef<[u8]>>(data: T) -> Result<Vec<Self>, Iso15765Error> {
        standard::from_data(data.as_ref())
    }

    #[inline]
    pub fn single_frame<T: AsRef<[u8]>>(data: T) -> Result<Self, Iso15765Error> {
        standard::new_single(data)
    }

    #[inline]
    pub fn flow_ctrl_frame(state: FlowControlState,
                           block_size: u8,
                           st_min: u8,
    ) -> Result<Self, Iso15765Error> {
        Ok(Self::FlowControlFrame(
            FlowControlContext::new(state, block_size, st_min)?
        ))
    }

    #[inline]
    pub fn default_flow_ctrl_frame() -> Self {
        Self::flow_ctrl_frame(
            FlowControlState::Continues,
            ISO_TP_DEFAULT_BLOCK_SIZE,
            ISO_TP_DEFAULT_ST_MIN
        )
            .unwrap()
    }
}
