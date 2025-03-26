pub mod algo;

use crc::{Crc, CRC_32_BZIP2};
use zlgcan_rs::{can::{ZCanChlMode, ZCanChlType}, device::ZCanDeviceType, driver::ZCanDriver, CHANNEL_MODE, CHANNEL_TYPE, DEVICE_INDEX, DEVICE_TYPE};
// use docan::{Client, DoCanError, DoCanClient, DoCanServer};
// use iso14229_1::{AddressAndLengthFormatIdentifier, RoutineCtrlType, TesterPresentType};
use rs_can::{CanError, DeviceBuilder, interfaces::ZLGCAN, ChannelConfig};

pub const CHANNEL: u8 = 0;

pub fn init_device() -> Result<ZCanDriver, CanError> {
    let dev_type = ZCanDeviceType::ZCAN_USBCANFD_200U;

    let mut builder = DeviceBuilder::new(ZLGCAN);

    let mut ch1_cfg = ChannelConfig::new(500_000);
    ch1_cfg.add_other(CHANNEL_MODE, Box::new(ZCanChlMode::Normal as u8))
        .add_other(CHANNEL_TYPE, Box::new(ZCanChlType::CAN as u8));

    let mut ch2_cfg = ChannelConfig::new(500_000);
    ch2_cfg.add_other(CHANNEL_MODE, Box::new(ZCanChlMode::Normal as u8))
        .add_other(CHANNEL_TYPE, Box::new(ZCanChlType::CAN as u8));

    builder.add_other(DEVICE_TYPE, Box::new(dev_type as u32))
        .add_other(DEVICE_INDEX, Box::new(0))
        .add_config(0.to_string(), ch1_cfg)
        .add_config(1.to_string(), ch2_cfg);

    let device = builder.build::<ZCanDriver>()?;

    Ok(device)
}

// pub fn init_client() -> Result<(
//     IsoTpAdapter<ZCanDriver, u8, CanMessage>,
//     DoCanClient<ZCanDriver, u8, CanMessage>,
// ), DoCanError> {
//     let driver = init_device()
//         .map_err(DoCanError::DeviceError)?;
//     std::thread::sleep(std::time::Duration::from_millis(100));
//     let mut adapter = IsoTpAdapter::new(driver);
//
//     let mut client = DoCanClient::new(adapter.clone(), Some(200));
//     client.init_channel(CHANNEL, Address {
//         tx_id: 0x7E0,
//         rx_id: 0x7E8,
//         fid: 0x7DF,
//     })?;
//
//     // let algo = Arc::new(Box::new(uds_security_algo));
//
//     adapter.start(100);
//
//     Ok((adapter, client))
// }
//
// pub fn init_server() -> Result<(
//     IsoTpAdapter<ZCanDriver, u8, CanMessage>,
//     DoCanServer<ZCanDriver, u8, CanMessage>,
// ), DoCanError> {
//     let driver = init_device()
//         .map_err(DoCanError::DeviceError)?;
//     std::thread::sleep(std::time::Duration::from_millis(100));
//     let mut adapter = IsoTpAdapter::new(driver);
//
//     let server = DoCanServer::new(adapter.clone(), 0, Address {
//         tx_id: 0x7E8,
//         rx_id: 0x7E0,
//         fid: 0x7DF,
//     });
//
//     adapter.start(100);
//
//     Ok((adapter, server))
// }

// pub fn uds_flash_file(
//     filepath: &str,
//     client: &mut DoCanClient<ZCanDriver, u8, CanMessage>,
//     channel: u8,
//     erase: bool,
// ) -> anyhow::Result<()> {
//     let bin_file = bin_file::BinFile::from_file(filepath)?;
//
//     if erase {
//         client.routine_control(channel, RoutineCtrlType::StartRoutine, 0xff00, vec![0; 8])?;
//     }
//
//     let mut data = Vec::new();
//     for segment in bin_file.segments() {
//         let mut count = 1;
//         let mut sequence = 1;
//         let segment_data = segment.data();
//         let length = segment_data.len();
//         data.extend_from_slice(segment_data);
//
//         let response = client.request_download(
//             channel,
//             AddressAndLengthFormatIdentifier::new(4, 4)?,
//             segment.minimum_address() as u128,
//             length as u128,
//             None,
//         )?;
//
//         let block_size = response.max_num_of_block_len as usize - 2;
//         println!("length: {}", length);
//
//         loop {
//             println!("count: {}, sequence: {}", count, sequence);
//             // std::io::stdout().flush().unwrap();
//
//             if count * block_size > length {
//                 let temp = segment_data[block_size * (count-1)..].to_vec();
//                 // println!("{}", hex_slice_to_string(temp.as_slice()));
//                 client.transfer_data(channel, sequence, temp)?;
//                 break;
//             }
//
//             let temp = segment_data[block_size*(count-1)..block_size*count].to_vec();
//             // println!("{}", hex_slice_to_string(temp.as_slice()));
//             client.transfer_data(channel, sequence, temp)?;
//
//             count += 1;
//             sequence = sequence.wrapping_add(1);
//
//             client.tester_present(channel, TesterPresentType::Zero, true, AddressType::Functional)?;
//         }
//
//         client.request_transfer_exit(channel, vec![])?;
//     }
//
//     let crc32 = Crc::<u32>::new(&CRC_32_BZIP2)
//         .checksum(data.as_slice());
//     let mut crc32_data = Vec::new();
//
//     for i in 0..4 {
//         let temp = (crc32 & (0xff000000 >> (8 * i))) >> (8 * (3 - i));
//         crc32_data.push(temp as u8);
//     }
//
//     client.routine_control(channel, RoutineCtrlType::StartRoutine, 0xff0a, crc32_data)?;
//
//     Ok(())
// }

