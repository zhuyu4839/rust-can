//! from [socketcan](https://crates.io/crates/socketcan)

use std::{ffi::CString, fmt, io, mem, os::raw::{c_int, c_void}, time::Duration, ptr};
use libc::*;

/// Tries to open the CAN socket by the interface number.
pub fn raw_open_socket(addr: &CanAddr) -> io::Result<c_int> {
    let fd = unsafe { socket(PF_CAN, SOCK_RAW, CAN_RAW) };

    if fd == -1 {
        return Err(io::Error::last_os_error());
    }

    let ret = unsafe { bind(fd, addr.as_sockaddr_ptr(), CanAddr::len() as u32) };

    if ret == -1 {
        let err = io::Error::last_os_error();
        unsafe { close(fd) };
        Err(err)
    } else {
        Ok(fd)
    }
}

// Enable or disable FD mode on the socket, fd.
pub fn set_fd_mode(fd: c_int, enable: bool) -> io::Result<c_int> {
    let enable = enable as c_int;

    let ret = unsafe {
        setsockopt(
            fd,
            SOL_CAN_RAW,
            CAN_RAW_FD_FRAMES,
            &enable as *const _ as *const c_void,
            mem::size_of::<c_int>() as u32,
        )
    };

    if ret == -1 {
        Err(io::Error::last_os_error())
    } else {
        Ok(fd)
    }
}

// Write a single frame of any type to the socket, fd.
pub fn raw_write_frame<T>(fd: c_int, frame_ptr: *const T, n: usize) -> io::Result<()> {
    let ret = unsafe { write(fd, frame_ptr.cast(), n) };

    if ret as usize == n {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

/// `setsockopt` wrapper
///
/// The libc `setsockopt` function is set to set various options on a socket.
/// `set_socket_option` offers a somewhat type-safe wrapper that does not
/// require messing around with `*const c_void`s.
///
/// A proper `std::io::Error` will be returned on failure.
///
/// Example use:
///
/// ```text
/// let fd = ...;  // some file descriptor, this will be stdout
/// set_socket_option(fd, SOL_TCP, TCP_NO_DELAY, 1 as c_int)
/// ```
///
/// Note that the `val` parameter must be specified correctly; if an option
/// expects an integer, it is advisable to pass in a `c_int`, not the default
/// of `i32`.
#[inline]
pub fn set_socket_option<T>(fd: c_int, level: c_int, name: c_int, val: &T) -> io::Result<()> {
    let ret = unsafe {
        setsockopt(
            fd,
            level,
            name,
            val as *const _ as *const c_void,
            mem::size_of::<T>() as socklen_t,
        )
    };

    if ret != 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(())
}

/// Sets a collection of multiple socke options with one call.
pub fn set_socket_option_mult<T>(
    fd: c_int,
    level: c_int,
    name: c_int,
    values: &[T],
) -> io::Result<()> {
    let ret = if values.is_empty() {
        // can't pass in a ptr to a 0-len slice, pass a null ptr instead
        unsafe { setsockopt(fd, level, name, ptr::null(), 0) }
    } else {
        unsafe {
            setsockopt(
                fd,
                level,
                name,
                values.as_ptr().cast(),
                mem::size_of_val(values) as socklen_t,
            )
        }
    };

    if ret != 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(())
}

// ===== can_frame =====

/// Creates a default C `can_frame`.
/// This initializes the entire structure to zeros.
#[inline(always)]
pub fn can_frame_default() -> can_frame {
    unsafe { mem::zeroed() }
}

/// Creates a default C `can_frame`.
/// This initializes the entire structure to zeros.
#[inline(always)]
pub fn canfd_frame_default() -> canfd_frame {
    unsafe { mem::zeroed() }
}

/// Check an error return value for timeouts.
///
/// Due to the fact that timeouts are reported as errors, calling `read_frame`
/// on a socket with a timeout that does not receive a frame in time will
/// result in an error being returned. This trait adds a `should_retry` method
/// to `Error` and `Result` to check for this condition.
pub trait ShouldRetry {
    /// Check for timeout
    ///
    /// If `true`, the error is probably due to a timeout.
    fn should_retry(&self) -> bool;
}

impl ShouldRetry for io::Error {
    fn should_retry(&self) -> bool {
        match self.kind() {
            // EAGAIN, EINPROGRESS and EWOULDBLOCK are the three possible codes
            // returned when a timeout occurs. the stdlib already maps EAGAIN
            // and EWOULDBLOCK os WouldBlock
            io::ErrorKind::WouldBlock => true,
            // however, EINPROGRESS is also valid
            io::ErrorKind::Other => {
                matches!(self.raw_os_error(), Some(errno) if errno == EINPROGRESS)
            }
            _ => false,
        }
    }
}

impl<E: fmt::Debug> ShouldRetry for io::Result<E> {
    fn should_retry(&self) -> bool {
        if let Err(ref e) = *self {
            e.should_retry()
        } else {
            false
        }
    }
}

/// CAN socket address.
///
/// This is the address for use with CAN sockets. It is simply an addres to
/// the SocketCAN host interface. It can be created by looking up the name
/// of the interface, like "can0", "vcan0", etc, or an interface index can
/// be specified directly, if known. An index of zero can be used to read
/// frames from all interfaces.
///
/// This is based on, and compatible with, the `sockaddr_can` struct from
/// libc.
/// [ref](https://docs.rs/libc/latest/libc/struct.sockaddr_can.html)
#[derive(Clone, Copy)]
pub struct CanAddr(sockaddr_can);

impl CanAddr {
    /// Creates a new CAN socket address for the specified interface by index.
    /// An index of zero can be used to read from all interfaces.
    pub fn new(ifindex: u32) -> Self {
        let mut addr = Self::default();
        addr.0.can_ifindex = ifindex as c_int;
        addr
    }

    /// Try to create an address from an interface name.
    pub fn from_iface(ifname: &str) -> io::Result<Self> {
        let ifname = CString::new(ifname)?;
        let ifindex = unsafe { if_nametoindex(ifname.as_ptr()) };
        if ifindex == 0 {
            Err(io::Error::last_os_error())
        }
        else {
            Ok(Self::new(ifindex))
        }
    }

    /// Gets the address of the structure as a `sockaddr_can` pointer.
    pub fn as_ptr(&self) -> *const sockaddr_can {
        &self.0
    }

    /// Gets the address of the structure as a `sockaddr` pointer.
    pub fn as_sockaddr_ptr(&self) -> *const sockaddr {
        self.as_ptr().cast()
    }

    /// Gets the size of the address structure.
    pub fn len() -> usize {
        mem::size_of::<sockaddr_can>()
    }

    /// Converts the CAN address into a `sockaddr_storage` type.
    /// This is a generic socket address container with enough space to hold
    /// any address type in the system.
    pub fn into_storage(self) -> (sockaddr_storage, socklen_t) {
        let mut storage: sockaddr_storage = unsafe { mem::zeroed() };
        unsafe {
            ptr::copy_nonoverlapping(&self.0, &mut storage as *mut _ as *mut sockaddr_can, 1);
        }
        (storage, Self::len() as socklen_t)
    }
}

impl Default for CanAddr {
    fn default() -> Self {
        let mut addr: sockaddr_can = unsafe { mem::zeroed() };
        addr.can_family = AF_CAN as sa_family_t;
        Self(addr)
    }
}

impl fmt::Debug for CanAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "CanAddr {{ can_family: {}, can_ifindex: {} }}",
            self.0.can_family, self.0.can_ifindex
        )
    }
}

impl From<sockaddr_can> for CanAddr {
    fn from(addr: sockaddr_can) -> Self {
        Self(addr)
    }
}

impl AsRef<sockaddr_can> for CanAddr {
    fn as_ref(&self) -> &sockaddr_can {
        &self.0
    }
}

pub fn c_timeval_new(t: Duration) -> timeval {
    timeval {
        tv_sec: t.as_secs() as time_t,
        tv_usec: t.subsec_micros() as suseconds_t,
    }
}
