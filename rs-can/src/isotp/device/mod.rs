mod adapter;
pub use adapter::*;
mod context;

use std::{any::Any, fmt::Display, sync::{Arc, Mutex, mpsc::Sender}, time::{Duration, Instant}, thread};
use iso15765_2::{FlowControlContext, FlowControlState, Iso15765Error, IsoTpEvent, IsoTpEventListener, IsoTpState, TIMEOUT_AS_ISO15765_2, TIMEOUT_CR_ISO15765_2};
use context::IsoTpContext;
use crate::{Frame, Id, Listener};
use crate::isotp::{Address, AddressType, IsoTpFrame, DEFAULT_P2_START_MS};

#[derive(Clone)]
pub struct CanIsoTp<C, F> {
    pub(crate) channel: C,
    pub(crate) address: Arc<Mutex<Address>>,
    pub(crate) sender: Sender<F>,
    pub(crate) context: Arc<Mutex<IsoTpContext>>,
    pub(crate) state: Arc<Mutex<IsoTpState>>,
    pub(crate) listener: Arc<Mutex<Box<dyn IsoTpEventListener>>>,
}

unsafe impl<C, F> Send for CanIsoTp<C, F> {}

impl<C: Clone, F: Frame<Channel = C>> CanIsoTp<C, F> {
    pub fn new(channel: C,
               address: Address,
               sender: Sender<F>,
               listener: Box<dyn IsoTpEventListener>,
    ) -> Self {
        Self {
            channel,
            address: Arc::new(Mutex::new(address)),
            sender,
            context: Default::default(),
            state: Default::default(),
            listener: Arc::new(Mutex::new(listener)),
        }
    }

    pub fn set_p2_context(&self, p2_ms: u16, p2_star_ms: u16) {
        match self.context.lock() {
            Ok(mut ctx) => {
                ctx.p2_ctx.update(p2_ms, p2_star_ms);
            },
            Err(e) =>
                log::warn!("CanIsoTp::set_p2_context: {}", e),
        }
    }

    #[inline]
    pub fn update_address(&self, address: Address) {
        if let Ok(mut addr) = self.address.lock() {
            *addr = address;
        }
    }

    pub fn write(&self, addr_type: AddressType, data: Vec<u8>) -> Result<(), Iso15765Error> {
        self.state_append(IsoTpState::Idle);
        self.context_reset();
        log::trace!("ISO-TP - Sending: {}", hex::encode(&data));

        let frames = IsoTpFrame::from_data(data)?;
        let frame_len = frames.len();

        let can_id = match self.address.lock() {
            Ok(address) => match addr_type {
                AddressType::Physical => Ok(address.tx_id),
                AddressType::Functional => Ok(address.fid)
            },
            Err(_) => {
                log::warn!("can't get address context");
                Err(Iso15765Error::DeviceError)
            },
        }?;
        let mut need_flow_ctrl = frame_len > 1;
        let mut index = 0;
        for frame in frames {
            let mut frame = F::from_iso_tp(can_id, frame, None)
                .ok_or({
                    log::warn!("fail to convert iso-tp frame to can frame");
                    Iso15765Error::DeviceError
                })?;
            frame.set_channel(self.channel.clone());

            if need_flow_ctrl {
                need_flow_ctrl = false;
                self.state_append(IsoTpState::Sending | IsoTpState::WaitFlowCtrl);
            }
            else {
                self.write_waiting(&mut index)?;
                self.state_append(IsoTpState::Sending);
            }
            self.sender.send(frame)
                .map_err(|e| {
                    log::warn!("ISO-TP - transmit failed: {:?}", e);
                    Iso15765Error::DeviceError
                })?;
        }

        Ok(())
    }

    #[inline]
    pub(crate) fn on_single_frame(&self, data: Vec<u8>) {
        self.iso_tp_event(IsoTpEvent::DataReceived(data));
    }

    #[inline]
    pub(crate) fn on_first_frame(&self, tx_id: u32, length: u32, data: Vec<u8>) {
        self.update_consecutive(length, data);

        let iso_tp_frame = IsoTpFrame::default_flow_ctrl_frame();
        match F::from_iso_tp(tx_id, iso_tp_frame, None) {
            Some(mut frame) => {
                frame.set_channel(self.channel.clone());

                self.state_append(IsoTpState::Sending);
                match self.sender.send(frame) {
                    Ok(_) => {
                        self.iso_tp_event(IsoTpEvent::FirstFrameReceived);
                    },
                    Err(e) => {
                        log::warn!("ISO-TP - transmit failed: {:?}", e);
                        self.state_append(IsoTpState::Error);

                        self.iso_tp_event(IsoTpEvent::ErrorOccurred(Iso15765Error::DeviceError));
                    },
                }
            },
            None => log::error!("ISO-TP - convert `iso-tp frame` to `can-frame` error"),
        }
    }

    #[inline]
    pub(crate) fn on_consecutive_frame(&self, sequence: u8, data: Vec<u8>) {
        match self.append_consecutive(sequence, data) {
            Ok(event) => self.iso_tp_event(event),
            Err(e) => {
                self.state_append(IsoTpState::Error);
                self.iso_tp_event(IsoTpEvent::ErrorOccurred(e));
            }
        }
    }

    #[inline]
    pub(crate) fn on_flow_ctrl_frame(&self, ctx: FlowControlContext) {
        match ctx.state() {
            FlowControlState::Continues => {
                self.state_remove(IsoTpState::WaitBusy | IsoTpState::WaitFlowCtrl);
            },
            FlowControlState::Wait => {
                self.state_append(IsoTpState::WaitBusy);
                self.iso_tp_event(IsoTpEvent::Wait);
                return;
            }
            FlowControlState::Overload => {
                self.state_append(IsoTpState::Error);
                self.iso_tp_event(IsoTpEvent::ErrorOccurred(Iso15765Error::OverloadFlow));
                return;
            }
        }

        if let Ok(mut context) = self.context.lock() {
            context.update_flow_ctrl(ctx);
        };
    }

    fn iso_tp_event(&self, event: IsoTpEvent) {
        match self.listener.lock() {
            Ok(mut listener) => {
                // println!("ISO-TP - Sending iso-tp event: {:?}", event);
                match &event {
                    IsoTpEvent::DataReceived(data) => {
                        log::debug!("ISO-TP - Received: {}", hex::encode(data));
                    },
                    IsoTpEvent::ErrorOccurred(_) =>
                        log::warn!("ISO-TP - Sending iso-tp event: {:?}", event),
                    _ => log::trace!("ISO-TP - Sending iso-tp event: {:?}", event),
                }
                listener.on_iso_tp_event(event);
            },
            Err(_) => log::warn!("ISO-TP(CAN async): Sending event failed"),
        }
    }

    fn write_waiting(&self, index: &mut usize) -> Result<(), Iso15765Error> {
        match self.context.lock() {
            Ok(ctx) => {
                if let Some(ctx) = &ctx.flow_ctrl {
                    if ctx.block_size != 0 {
                        if (*index + 1) == ctx.block_size as usize {
                            *index = 0;
                            self.state_append(IsoTpState::WaitFlowCtrl);
                        }
                        else {
                            *index += 1;
                        }
                    }
                    thread::sleep(Duration::from_micros(ctx.st_min as u64));
                }

                Ok(())
            },
            Err(_) => {
                log::warn!("can't get `context`");
                Err(Iso15765Error::DeviceError)
            }
        }?;

        let start = Instant::now();
        loop {
            if self.state_contains(IsoTpState::Error) {
                return Err(Iso15765Error::DeviceError);
            }

            if self.state_contains(IsoTpState::Sending) {
                if start.elapsed() > Duration::from_millis(TIMEOUT_AS_ISO15765_2 as u64) {
                    return Err(Iso15765Error::Timeout { value: TIMEOUT_AS_ISO15765_2 as u64, unit: "ms" });
                }
            }
            else if self.state_contains(IsoTpState::WaitBusy) {
                let p2_star = match self.context.lock() {
                    Ok(ctx) => {
                        ctx.p2_ctx.p2_star_ms()
                    },
                    Err(_) => DEFAULT_P2_START_MS,
                };
                if start.elapsed() > Duration::from_millis(p2_star) {
                    return Err(Iso15765Error::Timeout { value: p2_star, unit: "ms" });
                }
            }
            else if self.state_contains(IsoTpState::WaitFlowCtrl) {
                if start.elapsed() > Duration::from_millis(TIMEOUT_CR_ISO15765_2 as u64) {
                    return Err(Iso15765Error::Timeout { value: TIMEOUT_CR_ISO15765_2 as u64, unit: "ms" });
                }
            }
            else {
                break;
            }
        }

        Ok(())
    }

    fn append_consecutive(&self, sequence: u8, data: Vec<u8>) -> Result<IsoTpEvent, Iso15765Error> {
        match self.context.lock() {
            Ok(mut context) => {
                context.append_consecutive(sequence, data)
            },
            Err(_) => {
                log::warn!("can't get `context`");
                Err(Iso15765Error::DeviceError)
            }
        }
    }

    fn update_consecutive(&self, length: u32, data: Vec<u8>) {
        if let Ok(mut context) = self.context.lock() {
            context.update_consecutive(length, data);
        }
    }

    fn context_reset(&self) {
        if let Ok(mut context) = self.context.lock() {
            context.reset();
        };
    }

    #[inline]
    fn state_contains(&self, flags: IsoTpState) -> bool {
        match self.state.lock() {
            Ok(v) => {
                // log::debug!("ISO-TP - current state(state contains): {} contains: {}", *v, flags);
                *v & flags != IsoTpState::Idle
            },
            Err(_) => {
                log::warn!("ISO-TP - state mutex is poisoned");
                false
            },
        }
    }

    #[inline]
    fn state_append(&self, flags: IsoTpState) {
        match self.state.lock() {
            Ok(mut v) => {
                if flags == IsoTpState::Idle {
                    *v = IsoTpState::Idle;
                } else if flags.contains(IsoTpState::Error) {
                    *v = IsoTpState::Error;
                }
                else {
                    *v |= flags;
                }

                log::trace!("ISO-TP - current state(state append): {}", *v);
            }
            Err(_) => log::warn!("ISO-TP - state mutex is poisoned when appending"),
        }
    }

    #[inline]
    fn state_remove(&self, flags: IsoTpState) {
        match self.state.lock() {
            Ok(mut v) => {
                v.remove(flags);
                log::trace!("ISO-TP - current state(state remove): {}", *v);
            },
            Err(_) =>log::warn!("ISO-TP - state mutex is poisoned when removing"),
        }
    }
}

impl<C, F> Listener<C, F> for CanIsoTp<C, F>
where
    C: Clone + Eq + Display + 'static,
    F: Frame<Channel = C> + Clone + Display + 'static
{

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn on_frame_transmitting(&self, _: C, _: &F) {

    }

    fn on_frame_transmitted(&self, channel: C, id: Id) {
        let id = id.into_bits();
        log::trace!("ISO-TP transmitted: {:04X} from {}", id, channel);
        if channel != self.channel {
            return;
        }

        if let Ok(address) = self.address.lock() {
            if id == address.tx_id ||
                id == address.fid {
                self.state_remove(IsoTpState::Sending);
            }
        }
    }

    fn on_frame_received(&self, channel: C, frames: &[F]) {
        if channel != self.channel
            || self.state_contains(IsoTpState::Error) {
            return;
        }

        let address_id = if let Ok(address) = self.address.lock() {
            Some((address.tx_id, address.rx_id))
        }
        else {
            None
        };

        if let Some(address) = address_id {
            for frame in frames {
                if frame.id().into_bits() == address.1 {
                    log::debug!("ISO-TP received: {}", frame);

                    match IsoTpFrame::decode(frame.data()) {
                        Ok(frame) => match frame {
                            IsoTpFrame::SingleFrame { data } => {
                                self.on_single_frame(data);
                            }
                            IsoTpFrame::FirstFrame { length, data } => {
                                self.on_first_frame(address.0, length, data);
                            }
                            IsoTpFrame::ConsecutiveFrame { sequence, data } => {
                                self.on_consecutive_frame(sequence, data);
                            },
                            IsoTpFrame::FlowControlFrame(ctx) => {
                                self.on_flow_ctrl_frame(ctx);
                            },
                        },
                        Err(e) => {
                            log::warn!("ISO-TP - data convert to frame failed: {}", e);
                            self.state_append(IsoTpState::Error);
                            self.iso_tp_event(IsoTpEvent::ErrorOccurred(e));

                            break;
                        }
                    }
                }
            }
        }
    }
}
