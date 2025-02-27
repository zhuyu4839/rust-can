#![allow(unused_imports)]

use iso15765_2::{FrameType, Iso15765Error, ISO_TP_MAX_LENGTH_2004};
use crate::{CANFD_FRAME_MAX_SIZE, CAN_FRAME_MAX_SIZE, DEFAULT_PADDING};
use crate::isotp::{constant::{CONSECUTIVE_FRAME_SIZE, FIRST_FRAME_SIZE_2004, SINGLE_FRAME_SIZE_2004}, IsoTpFrame};
use super::parse_frame_util as parse;

#[cfg(feature = "isotp-can-fd")]
use crate::utils::can_dlc;

pub(crate) fn decode_single(data: &[u8],
                            byte0: u8,
                            length: usize
) -> Result<IsoTpFrame, Iso15765Error> {
    #[cfg(feature = "isotp-can-fd")]
    let max_len = CANFD_FRAME_MAX_SIZE;
    #[cfg(not(feature = "isotp-can-fd"))]
    let max_len = CAN_FRAME_MAX_SIZE;

    if length > max_len {
        return Err(Iso15765Error::LengthOutOfRange(length));
    }

    let pdu_len = byte0 & 0x0F;
    if length < pdu_len as usize + 1 {
        return Err(Iso15765Error::InvalidPdu(data.to_vec()));
    }

    Ok(IsoTpFrame::SingleFrame { data: Vec::from(&data[1..=pdu_len as usize]) })
}

pub(crate) fn decode_first(data: &[u8],
                           byte0: u8,
                           length: usize,
) -> Result<IsoTpFrame, Iso15765Error> {
    #[cfg(not(feature = "isotp-can-fd"))]
    if length != CAN_FRAME_MAX_SIZE {
        return Err(Iso15765Error::InvalidDataLength { actual: length, expect: CAN_FRAME_MAX_SIZE })
    }
    #[cfg(feature = "isotp-can-fd")]
    if length != CANFD_FRAME_MAX_SIZE {
        return Err(Iso15765Error::InvalidDataLength { actual: length, expect: CANFD_FRAME_MAX_SIZE })
    }

    let pdu_len = (byte0 as u16 & 0x0F) << 8 | data[1] as u16;
    Ok(IsoTpFrame::FirstFrame { length: pdu_len as u32, data: Vec::from(&data[2..]) })
}

pub(crate) fn encode_single(mut data: Vec<u8>, padding: Option<u8>) -> Vec<u8> {
    let length = data.len();
    let mut result = vec![FrameType::Single as u8 | length as u8];
    result.append(&mut data);
    #[cfg(not(feature = "isotp-can-fd"))]
    result.resize(CAN_FRAME_MAX_SIZE, padding.unwrap_or(DEFAULT_PADDING));
    #[cfg(feature = "isotp-can-fd")]
    if let Some(resize) = can_dlc(length, true) {
        result.resize(resize, padding.unwrap_or(DEFAULT_PADDING));
    }

    result
}

pub(crate) fn encode_first(length: u32, mut data: Vec<u8>) -> Vec<u8> {
    let len_h = ((length & 0x0F00) >> 8) as u8;
    let len_l = (length & 0x00FF) as u8;
    let mut result = vec![FrameType::First as u8 | len_h, len_l];
    result.append(&mut data);
    result
}

pub(crate) fn new_single<T: AsRef<[u8]>>(data: T) -> Result<IsoTpFrame, Iso15765Error> {
    let data = data.as_ref();
    let length = data.len();
    match length {
        0 => Err(Iso15765Error::EmptyPdu),
        1..=SINGLE_FRAME_SIZE_2004 => {
            let mut result = vec![FrameType::Single as u8 | length as u8];
            result.append(&mut data.to_vec());
            result.resize(SINGLE_FRAME_SIZE_2004, DEFAULT_PADDING);
            Ok(IsoTpFrame::SingleFrame { data: result })
        },
        v => Err(Iso15765Error::LengthOutOfRange(v)),
    }
}

pub(crate) fn from_data(data: &[u8]) -> Result<Vec<IsoTpFrame>, Iso15765Error> {
    let length = data.len();
    match length {
        0 => Err(Iso15765Error::EmptyPdu),
        1..=CONSECUTIVE_FRAME_SIZE => Ok(vec![IsoTpFrame::SingleFrame { data: data.to_vec() }]),
        ..=ISO_TP_MAX_LENGTH_2004 => {
            let mut offset = 0;
            let mut sequence = 1;
            let mut results = Vec::new();

            parse::<FIRST_FRAME_SIZE_2004>(data, &mut offset, &mut sequence, &mut results, length);

            Ok(results)
        },
        v => Err(Iso15765Error::LengthOutOfRange(v)),
    }
}
