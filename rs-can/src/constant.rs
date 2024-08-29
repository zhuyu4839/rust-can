use bitflags::bitflags;

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

/// Mask for standard identifiers.
pub const SFF_MASK: u32 = 0x0000_07FF;

/// Mask for extended identifiers.
pub const EFF_MASK: u32 = 0x1FFF_FFFF;
/// The max sizeof can-frame's data.
pub const CAN_FRAME_MAX_SIZE: usize = 8;
/// The max sizeof canfd-frame's data.
pub const CANFD_FRAME_MAX_SIZE: usize = 64;
/// Default padding value(0b1010_1010).
pub const DEFAULT_PADDING: u8 = 0xAA;
