mod identifier;
pub use identifier::*;

use std::fmt::{Display, Formatter, Write};
use crate::utils::can_dlc;

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Type {
    #[default]
    Can,
    CanFd,
    CanXl,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Direct {
    #[default]
    Transmit,
    Receive,
}

impl Display for Direct {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Transmit => f.write_str("Tx"),
            Self::Receive => f.write_str("Rx"),
        }
    }
}

/// CAN 2.0 | CAN 1.0
pub trait Frame: Send + Sync {
    type Channel: Display;

    fn new(id: impl Into<Id>, data: &[u8]) -> Option<Self>
    where
        Self: Sized;

    fn new_remote(id: impl Into<Id>, len: usize) -> Option<Self>
    where
        Self: Sized;

    fn timestamp(&self) -> u64;

    fn set_timestamp(&mut self, value: Option<u64>) -> &mut Self
    where
        Self: Sized;

    /// Prioritizes returning J1939Id if j1939 is true.
    fn id(&self) -> Id;

    fn can_type(&self) -> Type;

    fn set_can_type(&mut self, r#type: Type) -> &mut Self
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

    fn dlc(&self) -> isize {
        can_dlc(self.length(), self.can_type())
    }

    fn length(&self) -> usize;
}

impl<T: Display> Display for dyn Frame<Channel = T> {
    /// Output Frame as `asc` String.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let data_str = if self.is_remote() {
            " ".to_owned()
        } else {
            self.data()
                .iter()
                .fold(String::new(), |mut out, &b| {
                    let _ = write!(out, "{b:02x} ");
                    out
                })
        };

        match self.can_type() {
            Type::Can => {
                write!(f, "{:.3} {} {}{: <4} {} {} {} {}",
                       self.timestamp() as f64 / 1000.,
                       self.channel(),
                       format!("{: >8x}", self.id().into_bits()),
                       if self.is_extended() { "x" } else { "" },
                       self.direct(),
                       // if self.is_rx() { "Rx" } else { "Tx" },
                       if self.is_remote() { "r" } else { "d" },
                       format!("{: >2}", self.length()),
                       data_str,
                )
            },
            Type::CanFd => {
                let mut flags = 1 << 12;
                write!(f, "{:.3} CANFD {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
                       self.timestamp() as f64 / 1000.,
                       self.channel(),
                       self.direct(),
                       // if self.is_rx() { "Rx" } else { "Tx" },
                       format!("{: >8x}", self.id().into_bits()),
                       if self.is_bitrate_switch() {
                           flags |= 1 << 13;
                           1
                       } else { 0 },
                       if self.is_esi() {
                           flags |= 1 << 14;
                           1
                       } else { 0 },
                       format!("{: >2}", self.dlc()),
                       format!("{: >2}", self.length()),
                       data_str,
                       format!("{: >8}", 0),       // message_duration
                       format!("{: <4}", 0),       // message_length
                       format!("{: >8x}", flags),
                       format!("{: >8}", 0),       // crc
                       format!("{: >8}", 0),       // bit_timing_conf_arb
                       format!("{: >8}", 0),       // bit_timing_conf_data
                       format!("{: >8}", 0),       // bit_timing_conf_ext_arb
                       format!("{: >8}", 0),       // bit_timing_conf_ext_data
                )
            },
            Type::CanXl => {    // TODO
                write!(f, "CANXL Frame")
            }
        }
    }
}
