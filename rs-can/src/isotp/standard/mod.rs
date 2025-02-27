#[cfg(feature = "isotp-std2004")]
mod std2004;
#[cfg(feature = "isotp-std2004")]
pub(crate) use std2004::*;
#[cfg(feature = "isotp-std2016")]
mod std2016;
#[cfg(feature = "isotp-std2016")]
pub(crate) use std2016::*;

use crate::isotp::{constant::CONSECUTIVE_FRAME_SIZE, IsoTpFrame};

fn parse_frame_util<const FIRST_FRAME_SIZE: usize>(
    data: &[u8],
    offset: &mut usize,
    sequence: &mut u8,
    results: &mut Vec<IsoTpFrame>,
    length: usize,
) {
    loop {
        match *offset {
            0 => {
                *offset += FIRST_FRAME_SIZE;
                let frame = IsoTpFrame::FirstFrame {
                    length: length as u32,
                    data: Vec::from(&data[..*offset])
                };
                results.push(frame);
            },
            _ => {
                if *offset + CONSECUTIVE_FRAME_SIZE >= length {
                    let frame = IsoTpFrame::ConsecutiveFrame {
                        sequence: *sequence,
                        data: Vec::from(&data[*offset..length])
                    };
                    results.push(frame);
                    break;
                }

                let frame = IsoTpFrame::ConsecutiveFrame {
                    sequence: *sequence,
                    data: Vec::from(&data[*offset..*offset + CONSECUTIVE_FRAME_SIZE])
                };
                *offset += CONSECUTIVE_FRAME_SIZE;
                if *sequence >= 0x0F {
                    *sequence = 0;
                }
                else {
                    *sequence += 1;
                }

                results.push(frame);
            }
        }
    }
}
