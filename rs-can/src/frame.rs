use crate::constant::{EFF_MASK, SFF_MASK};
use isotp_rs::IsoTpFrame;
use std::fmt::{Display, Formatter, Write};

/// CAN direct define
#[repr(C)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub enum Direct {
    #[default]
    Transmit,
    Receive,
}

/// CAN filter define
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct CanFilter {
    pub can_id: u32,
    pub can_mask: u32,
    pub extended: bool,
}

/// CAN-Id define
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Id {
    Standard(u16),
    Extended(u32),
}

unsafe impl Send for Id {}

impl From<u32> for Id {
    #[inline]
    fn from(value: u32) -> Self {
        Self::from_bits(value, false)
    }
}

impl Into<u32> for Id {
    #[inline]
    fn into(self) -> u32 {
        self.into_bits()
    }
}

impl Id {
    #[inline]
    pub fn from_bits(bits: u32, extended: bool) -> Self {
        let bits = bits & EFF_MASK;
        if extended {
            Self::Extended(bits)
        } else {
            if bits & (!SFF_MASK & EFF_MASK) > 0 {
                Self::Extended(bits)
            } else {
                Self::Standard(bits as u16)
            }
        }
    }

    #[inline]
    pub fn from_hex(hex_str: &str, extended: bool) -> Option<Self> {
        let bits = u32::from_str_radix(hex_str, 16).ok()?;

        Some(Self::from_bits(bits, extended))
    }

    #[inline]
    pub fn try_from_bits(bits: u32, extended: bool) -> Option<Self> {
        match bits {
            0..=EFF_MASK => Some(Self::from_bits(bits, extended)),
            _ => None,
        }
    }

    #[inline]
    pub fn try_from_hex(hex_str: &str, extended: bool) -> Option<Self> {
        let value = u32::from_str_radix(hex_str, 16).ok()?;

        Self::try_from_bits(value, extended)
    }

    #[inline]
    pub fn into_bits(self) -> u32 {
        match self {
            Self::Standard(v) => v as u32,
            Self::Extended(v) => v,
        }
    }

    #[inline]
    pub fn into_hex(self) -> String {
        std::fmt::format(format_args!("{:08X}", self.into_bits()))
    }

    /// Returns this CAN Identifier as a raw 32-bit integer.
    #[inline]
    #[must_use]
    pub fn as_raw(self) -> u32 {
        self.into_bits()
    }

    /// Returns the Base ID part of this extended identifier.
    #[inline]
    #[must_use]
    pub fn standard_id(self) -> Self {
        match self {
            Self::Standard(_) => self.clone(),
            Self::Extended(v) => Self::Standard((v >> 18) as u16), // ID-28 to ID-18
        }
    }

    #[inline]
    pub fn is_extended(&self) -> bool {
        match self {
            Self::Standard(_) => false,
            Self::Extended(_) => true,
        }
    }
}

pub trait Frame {
    type Channel: Display;

    fn new(id: impl Into<Id>, data: &[u8]) -> Option<Self>
    where
        Self: Sized;

    fn new_remote(id: impl Into<Id>, len: usize) -> Option<Self>
    where
        Self: Sized;

    fn from_iso_tp(id: impl Into<Id>, frame: impl IsoTpFrame, padding: Option<u8>) -> Option<Self>
    where
        Self: Sized,
    {
        let data = frame.encode(padding);
        Self::new(id, data.as_slice())
    }

    fn timestamp(&self) -> u64;

    fn set_timestamp(&mut self, value: Option<u64>) -> &mut Self
    where
        Self: Sized;

    /// Prioritizes returning J1939Id if j1939 is true.
    fn id(&self) -> Id;

    fn is_can_fd(&self) -> bool;

    fn set_can_fd(&mut self, value: bool) -> &mut Self
    where
        Self: Sized;

    fn is_remote(&self) -> bool;

    fn is_extended(&self) -> bool;

    fn direct(&self) -> Direct;

    fn set_direct(&mut self, direct: Direct) -> &mut Self
    where
        Self: Sized;

    fn is_bitrate_switch(&self) -> bool;

    fn set_bitrate_switch(&mut self, value: bool) -> &mut Self
    where
        Self: Sized;

    fn is_error_frame(&self) -> bool;

    fn set_error_frame(&mut self, value: bool) -> &mut Self
    where
        Self: Sized;

    /// Error state indicator
    fn is_esi(&self) -> bool;

    /// Set error state indicator
    fn set_esi(&mut self, value: bool) -> &mut Self
    where
        Self: Sized;

    fn channel(&self) -> Self::Channel;

    fn set_channel(&mut self, value: Self::Channel) -> &mut Self
    where
        Self: Sized;

    /// ensure return the actual length of data.
    fn data(&self) -> &[u8];

    fn dlc(&self) -> Option<usize>;

    fn length(&self) -> usize;
}

impl<T: Display> Display for dyn Frame<Channel = T> {
    /// Output Frame as `asc` String.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let data_str = if self.is_remote() {
            " ".to_owned()
        } else {
            self.data().iter().fold(String::new(), |mut out, &b| {
                let _ = write!(out, "{b:02x} ");
                out
            })
        };

        if self.is_can_fd() {
            let mut flags = 1 << 12;
            write!(
                f,
                "{:.3} CANFD {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
                self.timestamp() as f64 / 1000.,
                self.channel(),
                direct(self.direct()),
                // if self.is_rx() { "Rx" } else { "Tx" },
                format!("{: >8x}", self.id().into_bits()),
                if self.is_bitrate_switch() {
                    flags |= 1 << 13;
                    1
                } else {
                    0
                },
                if self.is_esi() {
                    flags |= 1 << 14;
                    1
                } else {
                    0
                },
                format!("{: >2}", self.dlc().unwrap_or_default()),
                format!("{: >2}", self.length()),
                data_str,
                format!("{: >8}", 0), // message_duration
                format!("{: <4}", 0), // message_length
                format!("{: >8x}", flags),
                format!("{: >8}", 0), // crc
                format!("{: >8}", 0), // bit_timing_conf_arb
                format!("{: >8}", 0), // bit_timing_conf_data
                format!("{: >8}", 0), // bit_timing_conf_ext_arb
                format!("{: >8}", 0), // bit_timing_conf_ext_data
            )
        } else {
            write!(
                f,
                "{:.3} {} {}{: <4} {} {} {} {}",
                self.timestamp() as f64 / 1000.,
                self.channel(),
                format!("{: >8x}", self.id().into_bits()),
                if self.is_extended() { "x" } else { "" },
                direct(self.direct()),
                // if self.is_rx() { "Rx" } else { "Tx" },
                if self.is_remote() { "r" } else { "d" },
                format!("{: >2}", self.length()),
                data_str,
            )
        }
    }
}

#[inline]
fn direct<'a>(direct: Direct) -> &'a str {
    match direct {
        Direct::Transmit => "Tx",
        Direct::Receive => "Rx",
    }
}
