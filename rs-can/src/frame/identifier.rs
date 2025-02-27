use bitflags::bitflags;
use crate::{EFF_MASK, SFF_MASK};

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
    pub fn new_standard(id: u32) -> Self {
        Self::Standard(id as u16)
    }
    #[inline]
    pub fn new_extended(id: u32) -> Self {
        Self::Extended(id)
    }
    #[inline]
    pub fn from_bits(bits: u32, extended: bool) -> Self {
        if bits <= EFF_MASK {
            if extended {
                Self::Extended(bits)
            }
            else {
                Self::Standard(bits as u16)
            }
        }
        else {
            if !extended {
                Self::Standard((bits & SFF_MASK) as u16)
            }
            else {
                Self::Extended(bits & EFF_MASK)
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
            Self::Extended(v) => Self::Standard((v >> 18) as u16),     // ID-28 to ID-18
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
