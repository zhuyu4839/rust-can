mod constants;
pub use constants::*;
mod frame;
pub use frame::*;
mod socket;
pub use socket::*;

use std::{collections::HashMap, io, sync::Arc, os::{fd::{AsRawFd, BorrowedFd, FromRawFd, OwnedFd}, raw::{c_int, c_void}}, time::{Instant, Duration}};
use libc::{can_filter, can_frame, canfd_frame, canxl_frame, fcntl, read, CAN_RAW_ERR_FILTER, CAN_RAW_FILTER, CAN_RAW_JOIN_FILTERS, CAN_RAW_LOOPBACK, CAN_RAW_RECV_OWN_MSGS, EINPROGRESS, F_GETFL, F_SETFL, O_NONBLOCK, SOL_CAN_RAW, SOL_SOCKET, SO_RCVTIMEO, SO_SNDTIMEO};
use rs_can::{CanDevice, CanError, CanFilter, CanDirect, CanFrame, CanResult, ERR_MASK, DeviceBuilder, interfaces};

pub(crate) const FRAME_SIZE: usize = std::mem::size_of::<can_frame>();
pub(crate) const FD_FRAME_SIZE: usize = std::mem::size_of::<canfd_frame>();
pub(crate) const XL_FRAME_SIZE: usize = std::mem::size_of::<canxl_frame>();

#[derive(Debug, Clone)]
pub struct SocketCan {
    sockets: Arc<HashMap<String, OwnedFd>>,
}

impl SocketCan {
    pub fn new() -> Self {
        Self { sockets: Default::default() }
    }

    pub fn init_channel(&mut self, channel: &str, canfd: bool) -> Result<(), CanError> {
        let addr = CanAddr::from_iface(channel)
            .map_err(|e| CanError::InitializeError(e.to_string()))?;

        let _ = raw_open_socket(&addr)
            .and_then(|fd| {
                set_fd_mode(fd, canfd)
            })
            .and_then(|fd| {
                Arc::get_mut(&mut self.sockets)
                    .ok_or(io::Error::last_os_error())?
                    .insert(channel.to_owned(), unsafe { OwnedFd::from_raw_fd(fd) });
                // Rc::get_mut(&mut self.sockets)
                //     .ok_or(io::Error::last_os_error())?
                //     .insert(channel.to_owned(), unsafe { OwnedFd::from_raw_fd(fd) });
                Ok(())
            })
            .map_err(|_| CanError::OperationError("device open failed".into()));

        Ok(())
    }

    pub fn read(&self, channel: &str) -> Result<CanMessage, CanError> {
        match self.sockets.get(channel) {
            Some(s) => {
                let mut buffer = [0; XL_FRAME_SIZE];

                let rd = unsafe { read(
                    s.as_raw_fd(),
                    &mut buffer as *mut _ as *mut c_void,
                    XL_FRAME_SIZE
                ) };
                match rd {
                    FRAME_SIZE => {
                        let frame = unsafe { *(&buffer as *const _ as *const can_frame) };
                        let mut frame = CanMessage::from(CanAnyFrame::from(frame));
                        frame.set_direct(CanDirect::Receive);
                        Ok(frame)
                    },
                    FD_FRAME_SIZE => {
                        let frame = unsafe { *(&buffer as *const _ as *const canfd_frame) };
                        let mut frame = CanMessage::from(CanAnyFrame::from(frame));
                        frame.set_direct(CanDirect::Receive);
                        Ok(frame)
                    },
                    XL_FRAME_SIZE => {
                        let frame = unsafe { *(&buffer as *const _ as *const canxl_frame) };
                        let mut frame = CanMessage::from(CanAnyFrame::from(frame));
                        frame.set_direct(CanDirect::Receive);
                        Ok(frame)
                    },
                    _ => Err(CanError::OperationError(io::Error::last_os_error().to_string()))
                }
            },
            None => Err(CanError::channel_not_opened(channel))
        }
    }

    /// Blocking read a single can frame with timeout.
    pub fn read_timeout(&self, channel: &str, timeout: Duration) -> Result<CanMessage, CanError> {
        match self.sockets.get(channel) {
            Some(s) => {
                use nix::poll::{poll, PollFd, PollFlags};
                let borrowed_fd = unsafe { BorrowedFd::borrow_raw(s.as_raw_fd()) };
                let pollfd = PollFd::new(borrowed_fd, PollFlags::POLLIN);

                match poll::<u16>(&mut [pollfd], timeout.as_millis() as u16)
                    .map_err(|e| CanError::OperationError(e.to_string()))?
                {
                    0 => Err(CanError::channel_timeout(channel)),
                    _ => self.read(channel),
                }
            },
            None => Err(CanError::channel_not_opened(channel)),
        }
    }

    pub fn write(&self, msg: CanMessage) -> Result<(), CanError> {
        let channel = msg.channel();
        match self.sockets.get(&channel) {
            Some(s) => {
                let frame: CanAnyFrame = msg.into();
                match frame {
                    CanAnyFrame::Normal(f) |
                    CanAnyFrame::Remote(f) |
                    CanAnyFrame::Error(f) => {
                        raw_write_frame(s.as_raw_fd(), &f, frame.size())
                            .map_err(|e| CanError::OtherError(e.to_string()))
                    }
                    CanAnyFrame::Fd(f) => {
                        raw_write_frame(s.as_raw_fd(), &f, frame.size())
                            .map_err(|e| CanError::OtherError(e.to_string()))
                    },
                    CanAnyFrame::Xl(f) => {
                        raw_write_frame(s.as_raw_fd(), &f, frame.size())
                            .map_err(|e| CanError::OtherError(e.to_string()))
                    },
                }
            },
            None => Err(CanError::channel_not_opened(channel))
        }
    }

    /// Blocking write a single can frame, retrying until it gets sent successfully.
    pub fn write_timeout(&self, msg: CanMessage, timeout: Duration) -> Result<(), CanError> {
        let channel = msg.channel();
        let frame: CanAnyFrame = msg.into();
        let start = Instant::now();
        while start.elapsed() < timeout {
            match self.sockets.get(&channel) {
                Some(s) => {
                    if let Err(e) = match frame {
                        CanAnyFrame::Normal(f) |
                        CanAnyFrame::Remote(f) |
                        CanAnyFrame::Error(f) => {
                            raw_write_frame(s.as_raw_fd(), &f, frame.size())
                        }
                        CanAnyFrame::Fd(f) => {
                            raw_write_frame(s.as_raw_fd(), &f, frame.size())
                        },
                        CanAnyFrame::Xl(f) => {
                            raw_write_frame(s.as_raw_fd(), &f, frame.size())
                        }
                    } {
                        match e.kind() {
                            io::ErrorKind::WouldBlock => {},
                            io::ErrorKind::Other =>
                                if !matches!(e.raw_os_error(), Some(errno) if errno == EINPROGRESS) {
                                    return Err(CanError::OperationError(e.to_string()));
                                }
                            _ => return Err(CanError::OperationError(e.to_string())),
                        }
                    }
                    else {
                        return Ok(());
                    }
                },
                None => return Err(CanError::channel_not_opened(channel))
            }
        }

        Err(CanError::channel_timeout(channel))
    }

    /// Change socket to non-blocking mode or back to blocking mode.
    pub fn set_nonblocking(&self, channel: &str, nonblocking: bool) -> Result<(), CanError> {
        match self.sockets.get(channel) {
            Some(s) => {
                // retrieve current flags
                let oldfl = unsafe { fcntl(s.as_raw_fd(), F_GETFL) };

                if oldfl == -1 {
                    return Err(CanError::OperationError(io::Error::last_os_error().to_string()));
                }

                let newfl = if nonblocking {
                    oldfl | O_NONBLOCK
                } else {
                    oldfl & !O_NONBLOCK
                };

                let ret = unsafe { fcntl(s.as_raw_fd(), F_SETFL, newfl) };

                if ret != 0 {
                    Err(CanError::OperationError(io::Error::last_os_error().to_string()))
                }
                else {
                    Ok(())
                }
            },
            None => Err(CanError::channel_not_opened(channel))
        }
    }

    /// Sets the read timeout on the socket
    ///
    /// For convenience, the result value can be checked using
    /// `ShouldRetry::should_retry` when a timeout is set.
    pub fn set_read_timeout(&self, channel: &str, duration: Duration) -> Result<(), CanError> {
        match self.sockets.get(channel) {
            Some(s) => {
                set_socket_option(
                    s.as_raw_fd(),
                    SOL_SOCKET,
                    SO_RCVTIMEO,
                    &c_timeval_new(duration),
                )
                    .map_err(|e| CanError::OperationError(e.to_string()))
            },
            None => Err(CanError::channel_not_opened(channel))
        }
    }

    /// Sets the write timeout on the socket
    pub fn set_write_timeout(&self, channel: &str, duration: Duration) -> Result<(), CanError> {
        match self.sockets.get(channel) {
            Some(s) => {
                set_socket_option(
                    s.as_raw_fd(),
                    SOL_SOCKET,
                    SO_SNDTIMEO,
                    &c_timeval_new(duration),
                )
                    .map_err(|e| CanError::OperationError(e.to_string()))
            },
            None => Err(CanError::channel_not_opened(channel))
        }
    }
}

impl SocketCan {

    /// Sets CAN ID filters on the socket.
    ///
    /// CAN packages received by SocketCAN are matched against these filters,
    /// only matching packets are returned by the interface.
    ///
    /// See `CanFilter` for details on how filtering works. By default, all
    /// single filter matching all incoming frames is installed.
    pub fn set_filters(&self, channel: &str, filters: &[CanFilter]) -> Result<(), CanError> {
        match self.sockets.get(channel) {
            Some(s) => {
                let filters: Vec<can_filter> = filters.iter()
                    .map(|&f| {
                        can_filter {
                            can_id: f.can_id,
                            can_mask: f.can_mask,
                        }
                    })
                    .collect();
                set_socket_option_mult(s.as_raw_fd(), SOL_CAN_RAW, CAN_RAW_FILTER, &filters)
                    .map_err(|e| CanError::OperationError(e.to_string()))
            },
            None => Err(CanError::channel_not_opened(channel)),
        }
    }

    /// Disable reception of CAN frames.
    ///
    /// Sets a completely empty filter; disabling all CAN frame reception.
    pub fn set_filter_drop_all(&self, channel: &str) -> Result<(), CanError> {
        match self.sockets.get(channel) {
            Some(s) => {
                let filters: &[CanFilter] = &[];
                set_socket_option_mult(s.as_raw_fd(), SOL_CAN_RAW, CAN_RAW_FILTER, filters)
                    .map_err(|e| CanError::OperationError(e.to_string()))
            }
            None => Err(CanError::channel_not_opened(channel)),
        }
    }

    /// Accept all frames, disabling any kind of filtering.
    ///
    /// Replace the current filter with one containing a single rule that
    /// accepts all CAN frames.
    #[inline(always)]
    pub fn set_filter_accept_all(&self, channel: &str) -> Result<(), CanError> {
        self.set_filters(channel, &[CanFilter::from((0, 0))])
    }

    /// Sets the error mask on the socket.
    ///
    /// By default (`ERR_MASK_NONE`) no error conditions are reported as
    /// special error frames by the socket. Enabling error conditions by
    /// setting `ERR_MASK_ALL` or another non-empty error mask causes the
    /// socket to receive notification about the specified conditions.
    pub fn set_error_filter(&self, channel: &str, mask: u32) -> Result<(), CanError> {
        match self.sockets.get(channel) {
            Some(s) => {
                set_socket_option(s.as_raw_fd(), SOL_CAN_RAW, CAN_RAW_ERR_FILTER, &mask)
                    .map_err(|e| CanError::OperationError(e.to_string()))
            }
            None => Err(CanError::channel_not_opened(channel)),
        }
    }

    /// Sets the error mask on the socket to reject all errors.
    #[inline(always)]
    pub fn set_error_filter_drop_all(&self, channel: &str) -> Result<(), CanError> {
        self.set_error_filter(channel, 0)
    }

    /// Sets the error mask on the socket to accept all errors.
    #[inline(always)]
    pub fn set_error_filter_accept_all(&self, channel: &str) -> Result<(), CanError> {
        self.set_error_filter(channel, ERR_MASK)
    }

    /// Enable or disable loopback.
    ///
    /// By default, loopback is enabled, causing other applications that open
    /// the same CAN bus to see frames emitted by different applications on
    /// the same system.
    pub fn set_loopback(&self, channel: &str, enabled: bool) -> Result<(), CanError> {
        match self.sockets.get(channel) {
            Some(s) => {
                let loopback = c_int::from(enabled);
                set_socket_option(s.as_raw_fd(), SOL_CAN_RAW, CAN_RAW_LOOPBACK, &loopback)
                    .map_err(|e| CanError::OperationError(e.to_string()))
            }
            None => Err(CanError::channel_not_opened(channel)),
        }
    }

    /// Enable or disable receiving of own frames.
    ///
    /// When loopback is enabled, this settings controls if CAN frames sent
    /// are received back immediately by sender. Default is off.
    pub fn set_recv_own_msgs(&self, channel: &str, enabled: bool) -> Result<(), CanError> {
        match self.sockets.get(channel) {
            Some(s) => {
                let recv_own_msgs = c_int::from(enabled);
                set_socket_option(
                    s.as_raw_fd(),
                    SOL_CAN_RAW,
                    CAN_RAW_RECV_OWN_MSGS,
                    &recv_own_msgs,
                )
                    .map_err(|e| CanError::OperationError(e.to_string()))
            }
            None => Err(CanError::channel_not_opened(channel)),
        }
    }

    /// Enable or disable join filters.
    ///
    /// By default a frame is accepted if it matches any of the filters set
    /// with `set_filters`. If join filters is enabled, a frame has to match
    /// _all_ filters to be accepted.
    pub fn set_join_filters(&self, channel: &str, enabled: bool) -> Result<(), CanError> {
        match self.sockets.get(channel) {
            Some(s) => {
                let join_filters = c_int::from(enabled);
                set_socket_option(
                    s.as_raw_fd(),
                    SOL_CAN_RAW,
                    CAN_RAW_JOIN_FILTERS,
                    &join_filters,
                )
                    .map_err(|e| CanError::OperationError(e.to_string()))
            }
            None => Err(CanError::OperationError(format!("channel {} not opened", channel))),
        }
    }
}

impl TryFrom<DeviceBuilder, Error=CanError> for SocketCan {
    type Error = CanError;

    fn try_from(builder: DeviceBuilder) -> Result<Self, Self::Error> {
        if builder.interface() != interfaces::SOCKETCAN {
            return Err(CanError::interface_not_matched(builder.interface()));
        }

        let mut device = SocketCan::new();
        builder.channel_configs()
            .iter()
            .try_for_each(|(clh, cfg)| {
                let canfd = builder.get_other::<bool>(CANFD)?
                    .unwrap_or_default();
                device.init_channel(clh, canfd)?;

                if let Some(filters) = builder.get_other::<Vec<CanFilter>>(FILTERS)? {
                    device.set_filters(clh, &filters)?;
                }

                if let Some(loopback) = builder.get_other::<bool>(LOOPBACK)? {
                    device.set_loopback(clh, loopback)?;
                }

                if let Some(recv_own_msg) = builder.get_other::<bool>(RECV_OWN_MSG) {
                    device.set_recv_own_msgs(clh, recv_own_msg)?;
                }

                Ok(())
            })?;

        Ok(device)
    }
}

impl CanDevice for SocketCan {
    type Channel = String;
    type Frame = CanMessage;

    #[inline(always)]
    fn opened_channels(&self) -> Vec<Self::Channel> {
        self.sockets.iter()
            .map(|(c, _)| c.clone())
            .collect()
    }

    #[inline(always)]
    fn transmit(&self, msg: Self::Frame, timeout: Option<u32>) -> CanResult<(), CanError> {
        match timeout {
            Some(timeout) => self.write_timeout(msg, Duration::from_millis(timeout as u64)),
            None => self.write(msg),
        }
    }

    #[inline(always)]
    fn receive(&self, channel: Self::Channel, timeout: Option<u32>) -> CanResult<Vec<Self::Frame>, CanError> {
        let timeout = timeout.unwrap_or(0);
        let msg = self.read_timeout(&channel, Duration::from_millis(timeout as u64))?;
        Ok(vec![msg, ])
    }

    #[inline(always)]
    fn shutdown(&mut self) {
        match Arc::get_mut(&mut self.sockets) {
            Some(s) => s.clear(),
            None => (),
        }
    }
}
