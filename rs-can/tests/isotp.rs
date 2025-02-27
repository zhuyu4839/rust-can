use iso15765_2::FlowControlState;
use rs_can::{{CAN_FRAME_MAX_SIZE, DEFAULT_PADDING}, isotp::{IsoTpFrame, CONSECUTIVE_FRAME_SIZE, FIRST_FRAME_SIZE_2004}};

#[test]
fn test_single() -> anyhow::Result<()> {
    let data = hex::decode("0210010000000000")?;
    let frame = IsoTpFrame::decode(&data)?;
    match frame.clone() {
        IsoTpFrame::SingleFrame { data } => {
            assert_eq!(data, hex::decode("1001")?);
        },
        _ => {
            panic!("Invalid frame type");
        }
    }
    assert_eq!(frame.encode(Some(0x00)), data.to_vec());

    let frame = IsoTpFrame::SingleFrame { data: hex::decode("1001")? };
    assert_eq!(frame.encode(Some(0x00)), data.to_vec());
    Ok(())
}

#[test]
fn test_first() -> anyhow::Result<()> {
    let data = hex::decode("100f62f187445643")?;
    let frame = IsoTpFrame::decode(&data)?;
    match frame.clone() {
        IsoTpFrame::FirstFrame { length, data } => {
            assert_eq!(length, 0x0f);
            assert_eq!(data, hex::decode("62f187445643")?);
        },
        _ => {
            panic!("Invalid frame type");
        }
    }
    assert_eq!(frame.encode(None), data.to_vec());

    let frame = IsoTpFrame::FirstFrame {
        length: 0x0f,
        data: hex::decode("62f187445643")?
    };
    assert_eq!(frame.encode(None), data.to_vec());

    Ok(())
}

#[test]
fn test_consecutive() -> anyhow::Result<()> {
    let data = hex::decode("2137453230303030")?;
    let frame = IsoTpFrame::decode(&data)?;
    match frame.clone() {
        IsoTpFrame::ConsecutiveFrame { sequence, data } => {
            assert_eq!(sequence, 1);
            assert_eq!(data, hex::decode("37453230303030")?);
        },
        _ => {
            panic!("Invalid frame type");
        }
    }
    assert_eq!(frame.encode(None), data.to_vec());

    let frame = IsoTpFrame::ConsecutiveFrame {
        sequence: 1,
        data: hex::decode("37453230303030")?
    };
    assert_eq!(frame.encode(None), data.to_vec());
    Ok(())
}

#[test]
fn test_flow_control() -> anyhow::Result<()> {
    let data = hex::decode("3080015555555555")?;
    let frame = IsoTpFrame::decode(&data)?;
    match frame.clone() {
        IsoTpFrame::FlowControlFrame(context) => {
            assert_eq!(context.state(), FlowControlState::Continues);
            assert_eq!(context.block_size(), 0x80);
            assert_eq!(context.st_min(), 0x01);
        },
        _ => {
            panic!("Invalid frame type");
        }
    }
    assert_eq!(frame.encode(Some(0x55)), data);

    let frame = IsoTpFrame::default_flow_ctrl_frame();
    assert_eq!(frame.encode(Some(0x55)), hex::decode("30000a5555555555")?);
    Ok(())
}

#[test]
fn test_data_to_multi() -> anyhow::Result<()> {
    let data = hex::decode("62f187445643374532303030303037")?;
    let frames = IsoTpFrame::from_data(data)?;
    for (index, frame) in frames.into_iter().enumerate() {
        match index {
            0 => {
                assert_eq!(frame.encode(None), hex::decode("100f62f187445643")?);
            },
            1 => {
                assert_eq!(frame.encode(None), hex::decode("2137453230303030")?);
            },
            2 => assert_eq!(frame.encode(None), hex::decode("223037aaaaaaaaaa")?),
            _ => panic!()
        }
    }

    let mut size = 0x96;
    let data = vec![0x30; size];
    let frames = IsoTpFrame::from_data(data)?;
    for (index, frame) in frames.into_iter().enumerate() {
        match index {
            0 => {
                size -= FIRST_FRAME_SIZE_2004;
                assert_eq!(frame.encode(None), hex::decode("1096303030303030")?)
            },
            1..=15 => {
                size -= CONSECUTIVE_FRAME_SIZE;
                let expect = vec![0x20 + index as u8, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30];
                assert_eq!(frame.encode(None), expect);
            }
            _ => {
                if size > CONSECUTIVE_FRAME_SIZE {
                    size -= CONSECUTIVE_FRAME_SIZE;
                    let expect = vec![0x20 + (index % 16) as u8, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30];
                    assert_eq!(frame.encode(None), expect);
                }
                else {
                    let mut expect = vec![0x20 + (index % 16) as u8];
                    for _ in 0..size {
                        expect.push(0x30);
                    }
                    expect.resize(CAN_FRAME_MAX_SIZE, DEFAULT_PADDING);
                    assert_eq!(frame.encode(None), expect);
                }
            },
        }
    }
    Ok(())
}
