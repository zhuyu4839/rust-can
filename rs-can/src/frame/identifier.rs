use bitflags::bitflags;
use crate::constants::{EFF_MASK, SFF_MASK};

bitflags! {
    /// Identifier flags for indicating various frame types.
    ///
    /// These flags are applied logically in `can`, but flag values themselves correspond to the
    /// format used by the Linux [SocketCAN][socketcan] library.  This lets flags be applied
    /// logically to identifiers such that callers can construct their calls to the underlying CAN
    /// transceivers/controllers in whatever way is required, but also provides a happy path for
    /// SocketCAN users by allowing generation of the all-in-one 32-bit identifier value.
    ///
    /// [socketcan]: https://www.kernel.org/doc/Documentation/networking/can.txt
    #[repr(transparent)]
    pub struct IdentifierFlags: u32 {
        /// The frame is using the extended format i.e. 29-bit extended identifiers.
        const EXTENDED = 0x8000_0000;
        /// The frame is a remote transmission request.
        const REMOTE = 0x4000_0000;
        /// The frame is an error frame.
        const ERROR = 0x2000_0000;
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Filter {
    pub can_id: u32,
    pub can_mask: u32,
    pub extended: bool
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Id {
    Standard(u16),
    Extended(u32),
}

unsafe impl Send for Id {}

impl Into<u32> for Id {
    fn into(self) -> u32 {
        match self {
            Self::Standard(id) => id as u32,
            Self::Extended(id) => id,
        }
    }
}

impl From<u32> for Id {
    fn from(id: u32) -> Self {
        Self::_from_bits(id)
    }
}

impl Id {
    #[inline]
    pub fn new_standard(id: u16) -> Self {
        Self::Standard(id)
    }

    #[inline]
    pub fn new_extended(id: u32) -> Self {
        Self::Extended(id)
    }

    #[inline]
    pub fn from_bits(id: u32, force_extend: Option<bool>) -> Self {
        match force_extend {
            Some(true) => Self::Extended(id),
            _ => Self::_from_bits(id)
        }
    }

    #[inline]
    pub fn into_bits(self) -> u32 {
        match self {
            Self::Standard(id) => id as u32,
            Self::Extended(id) => id,
        }
    }

    #[inline]
    pub fn from_hex(hex_str: &str, force_extend: Option<bool>) -> Option<Self> {
        let bits = u32::from_str_radix(hex_str, 16).ok()?;

        Some(Self::from_bits(bits, force_extend))
    }

    #[inline]
    pub fn into_hex(self) -> String {
        std::fmt::format(format_args!("{:08X}", self.into_bits()))
    }

    /// Returns the Base ID part of this extended identifier.
    #[inline]
    pub fn standard_id(self) -> Self {
        match self {
            Self::Standard(_) => self,
            Self::Extended(v) => Self::Standard((v >> 18) as u16),     // ID-28 to ID-18
        }
    }

    /// Returns this CAN Identifier as a raw 32-bit integer.
    #[inline]
    pub fn as_raw(self) -> u32 {
        self.into_bits()
    }

    #[inline]
    pub fn is_extended(&self) -> bool {
        matches!(self, Self::Extended(_))
    }

    #[inline]
    fn _from_bits(id: u32) -> Self {
        match id & IdentifierFlags::EXTENDED.bits() {
            0 => if id > SFF_MASK {
                Self::new_extended(id)
            } else {
                Self::new_standard(id as u16)
            }
            _ => Self::new_extended(id & EFF_MASK),
        }
    }
}
