use iso15765_2::{FlowControlContext, Iso15765Error, IsoTpEvent, ISO_TP_CONSECUTIVE_SEQUENCE_START};
use crate::isotp::P2Context;

#[derive(Debug, Default, Clone)]
pub(crate) struct FlowCtrl {
    pub(crate) st_min: u32,    // μs
    pub(crate) block_size: u8,
}

/// Consecutive frame data context.
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub(crate) struct Consecutive {
    pub(crate) sequence: Option<u8>,
    pub(crate) length: Option<u32>,
    pub(crate) buffer: Vec<u8>,
}

#[derive(Debug, Default, Clone)]
pub struct IsoTpContext {
    pub(crate) p2_ctx: P2Context,
    pub(crate) flow_ctrl: Option<FlowCtrl>,
    pub(crate) consecutive: Consecutive,
}

impl IsoTpContext {
    /// reset st_min/consecutive/block_size
    #[inline]
    pub(crate) fn reset(&mut self) {
        self.p2_ctx = Default::default();
        self.clear_flow_ctrl();
        self.clear_consecutive();
    }
    #[inline]
    pub(crate) fn clear_flow_ctrl(&mut self) {
        self.flow_ctrl = Default::default();
    }
    #[inline]
    pub(crate) fn update_flow_ctrl(&mut self, ctx: FlowControlContext) {
        self.flow_ctrl = Some(FlowCtrl {
            st_min: ctx.st_min_us(),
            block_size: ctx.block_size(),
        });
    }
    #[inline]
    pub(crate) fn clear_consecutive(&mut self) {
        self.consecutive.sequence = Default::default();
        self.consecutive.length = Default::default();
        self.consecutive.buffer.clear();
    }
    #[inline]
    pub(crate) fn update_consecutive(&mut self, length: u32, mut data: Vec<u8>) {
        self.consecutive.length = Some(length);
        self.consecutive.buffer.append(&mut data);
    }
    pub(crate) fn append_consecutive(&mut self, sequence: u8, mut data: Vec<u8>) -> Result<IsoTpEvent, Iso15765Error> {
        if self.consecutive.length.is_none() {
            return Err(Iso15765Error::MixFramesError);
        }

        let target = match self.consecutive.sequence {
            Some(v) => match v {
                ..=0x0E => v + 1,
                _ => 0,
            },
            None => ISO_TP_CONSECUTIVE_SEQUENCE_START
        };
        self.consecutive.sequence = Some(target);
        if sequence != target {
            return Err(Iso15765Error::InvalidSequence { expect: target, actual: sequence });
        }

        self.consecutive.buffer.append(&mut data);

        let buff_len = self.consecutive.buffer.len();
        let target_len = self.consecutive.length.unwrap() as usize;
        if buff_len >= target_len {
            self.consecutive.buffer.resize(target_len, 0);
            let data = self.consecutive.buffer.clone();
            Ok(IsoTpEvent::DataReceived(data))
        }
        else {
            Ok(IsoTpEvent::Wait)
        }
    }
}
