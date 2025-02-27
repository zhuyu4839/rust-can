#![allow(unused_imports)]

use iso15765_2::{FrameType, Iso15765Error, ISO_TP_MAX_LENGTH_2004, ISO_TP_MAX_LENGTH_2016};
use crate::{CANFD_FRAME_MAX_SIZE, CAN_FRAME_MAX_SIZE, DEFAULT_PADDING};
use crate::driver::isotp::constant::{FIRST_FRAME_SIZE_2004, FIRST_FRAME_SIZE_2016, SINGLE_FRAME_SIZE_2004, SINGLE_FRAME_SIZE_2016};
use crate::isotp::IsoTpFrame;
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

    let mut pdu_len = byte0 & 0x0F;
    return if pdu_len > 0 {
        if length < pdu_len as usize + 1 {
            return Err(Iso15765Error::InvalidPdu(data.to_vec()));
        }

        Ok(IsoTpFrame::SingleFrame { data: Vec::from(&data[1..=pdu_len as usize]) })
    } else {
        pdu_len = data[1];
        if length < pdu_len as usize + 2 {
            return Err(Iso15765Error::InvalidPdu(data.to_vec()));
        }
        Ok(IsoTpFrame::SingleFrame { data: Vec::from(&data[2..=pdu_len as usize]) })
    }
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

    let mut pdu_len = (byte0 as u32 & 0x0F) << 8 | data[1] as u32;
    if pdu_len > 0 {
        Ok(IsoTpFrame::FirstFrame { length: pdu_len, data: Vec::from(&data[2..]) })
    }
    else {
        pdu_len = u32::from_be_bytes([data[2], data[3], data[4], data[5]]);
        Ok(IsoTpFrame::FirstFrame { length: pdu_len, data: Vec::from(&data[6..]) })
    }
}

pub(crate) fn encode_single(mut data: Vec<u8>, padding: Option<u8>) -> Vec<u8> {
    let length = data.len();
    match length {
        ..=SINGLE_FRAME_SIZE_2004 => {
            let mut result = vec![FrameType::Single as u8 | length as u8];
            result.append(&mut data);
            #[cfg(not(feature = "isotp-can-fd"))]
            result.resize(CAN_FRAME_MAX_SIZE, padding.unwrap_or(DEFAULT_PADDING));
            #[cfg(feature = "isotp-can-fd")]
            if let Some(resize) = can_dlc(length, true) {
                result.resize(resize, padding.unwrap_or(DEFAULT_PADDING));
            }
            result
        },
        _ => {
            let mut result = vec![FrameType::Single as u8, length as u8];
            result.append(&mut data);
            #[cfg(not(feature = "isotp-can-fd"))]
            result.resize(CAN_FRAME_MAX_SIZE, padding.unwrap_or(DEFAULT_PADDING));
            #[cfg(feature = "isotp-can-fd")]
            if let Some(resize) = can_dlc(length, true) {
                result.resize(resize, padding.unwrap_or(DEFAULT_PADDING));
            }

            result
        }
    }
}

pub(crate) fn encode_first(length: u32, mut data: Vec<u8>) -> Vec<u8> {
    let mut result = if length & 0xFFFFFFFF > 0x7FF {
        let mut temp = vec![FrameType::First as u8];
        temp.extend(length.to_be_bytes());
        temp
    }
    else {
        let len_h = ((length & 0x0F00) >> 8) as u8;
        let len_l = (length & 0x00FF) as u8;
        vec![FrameType::First as u8 | len_h, len_l]
    };
    result.append(&mut data);
    result
}

pub(crate) fn new_single<T: AsRef<[u8]>>(data: T) -> Result<IsoTpFrame, Iso15765Error> {
    let data = data.as_ref();
    let length = data.len();
    match length {
        0 => Err(Iso15765Error::EmptyPdu),
        1..=SINGLE_FRAME_SIZE_2016 => {
            let mut result = vec![FrameType::Single as u8 | length as u8];
            result.append(&mut data.to_vec());
            result.resize(SINGLE_FRAME_SIZE_2016, DEFAULT_PADDING);
            Ok(IsoTpFrame::SingleFrame { data: result })
        },
        v => Err(Iso15765Error::LengthOutOfRange(v)),
    }
}

pub(crate) fn from_data(data: &[u8]) -> Result<Vec<IsoTpFrame>, Iso15765Error> {
    let length = data.len();
    match length {
        0 => Err(Iso15765Error::EmptyPdu),
        ..=SINGLE_FRAME_SIZE_2004 => Ok(vec![IsoTpFrame::SingleFrame { data: data.to_vec() }]),
        ..=ISO_TP_MAX_LENGTH_2004 => {
            let mut offset = 0;
            let mut sequence = 1;
            let mut results = Vec::new();

            parse::<FIRST_FRAME_SIZE_2004>(data, &mut offset, &mut sequence, &mut results, length);

            Ok(results)
        },
        ..=ISO_TP_MAX_LENGTH_2016 => {
            let mut offset = 0;
            let mut sequence = 1;
            let mut results = Vec::new();

            parse::<FIRST_FRAME_SIZE_2016>(data, &mut offset, &mut sequence, &mut results, length);

           Ok(results)
        },
        v => Err(Iso15765Error::LengthOutOfRange(v)),
    }
}
