mod algo;
pub(crate) use algo::uds_security_algo;

use crc::{Crc, CRC_32_BZIP2};
use isotp_rs::can::Address;
use isotp_rs::can::driver::SyncCan;
use zlgcan::can::{CanChlCfgExt, CanChlCfgFactory, CanMessage, ZCanChlMode, ZCanChlType};
use zlgcan::device::ZCanDeviceType;
use zlgcan::driver::{ZCanDriver, ZDevice};
use ecu_uds::docan::client::SyncClient;
use ecu_uds::error::Error;
use ecu_uds::service::{AddressAndLengthFormatIdentifier, RoutineCtrlType, TesterPresentType};

pub(crate) const CHANNEL: u8 = 0;

pub(crate) fn init_client() -> Result<(
    // ZCanDriver,
    SyncCan<ZCanDriver, u8, CanMessage>,
    SyncClient<ZCanDriver, u8, CanMessage>,
), Error> {
    let dev_type = ZCanDeviceType::ZCAN_USBCANFD_200U;
    let mut device = ZCanDriver::new(dev_type as u32, 0, None)
        .map_err(|e| Error::OtherError(e.to_string()))?;
    device.open()
        .map_err(|e| Error::OtherError(e.to_string()))?;

    let factory = CanChlCfgFactory::new()
        .map_err(|e| Error::OtherError(e.to_string()))?;
    let ch1_cfg = factory.new_can_chl_cfg(dev_type as u32, ZCanChlType::CAN as u8, ZCanChlMode::Normal as u8, 500_000,
                                          CanChlCfgExt::default())
        .map_err(|e| Error::OtherError(e.to_string()))?;
    let ch2_cfg = factory.new_can_chl_cfg(dev_type as u32, ZCanChlType::CAN as u8, ZCanChlMode::Normal as u8, 500_000,
                                          CanChlCfgExt::default())
        .map_err(|e| Error::OtherError(e.to_string()))?;
    let cfg = vec![ch1_cfg, ch2_cfg];
    device.init_can_chl(cfg)
        .map_err(|e| Error::OtherError(e.to_string()))?;

    std::thread::sleep(std::time::Duration::from_millis(100));

    let mut sync_can = SyncCan::new(device.clone());

    let mut client = SyncClient::new(sync_can.clone());
    client.init_channel(CHANNEL, Address {
        tx_id: 0x7E0,
        rx_id: 0x7E8,
        fid: 0x7DF,
    }, None)?;

    // let algo = Arc::new(Box::new(uds_security_algo));

    sync_can.sync_start(100);

    Ok((sync_can, client))
}

pub(crate) fn uds_flash_file(
    filepath: &str,
    client: &mut SyncClient<ZCanDriver, u8, CanMessage>,
    channel: u8,
    erase: bool,
) -> anyhow::Result<()> {
    let bin_file = bin_file::BinFile::from_file(filepath)?;

    if erase {
        client.routine_control(channel, RoutineCtrlType::StartRoutine, 0xff00, vec![0; 8])?;
    }

    let mut data = Vec::new();
    for segment in bin_file.segments() {
        let mut count = 1;
        let mut sequence = 1;
        let segment_data = segment.data();
        let length = segment_data.len();
        data.extend_from_slice(segment_data);

        let response = client.request_download(
            channel,
            AddressAndLengthFormatIdentifier::new(4, 4)?,
            segment.minimum_address() as u128,
            length as u128,
            None,
        )?;

        let block_size = response.max_num_of_block_len as usize - 2;
        println!("length: {}", length);

        loop {
            println!("count: {}, sequence: {}", count, sequence);
            // std::io::stdout().flush().unwrap();

            if count * block_size > length {
                let temp = segment_data[block_size * (count-1)..].to_vec();
                // println!("{}", hex_slice_to_string(temp.as_slice()));
                client.transfer_data(channel, sequence, temp)?;
                break;
            }

            let temp = segment_data[block_size*(count-1)..block_size*count].to_vec();
            // println!("{}", hex_slice_to_string(temp.as_slice()));
            client.transfer_data(channel, sequence, temp)?;

            count += 1;
            sequence = sequence.wrapping_add(1);

            client.tester_present(channel, TesterPresentType::Zero, true, true)?;
        }

        client.request_transfer_exit(channel, vec![])?;
    }

    let crc32 = Crc::<u32>::new(&CRC_32_BZIP2)
        .checksum(data.as_slice());
    let mut crc32_data = Vec::new();

    for i in 0..4 {
        let temp = (crc32 & (0xff000000 >> (8 * i))) >> (8 * (3 - i));
        crc32_data.push(temp as u8);
    }

    client.routine_control(channel, RoutineCtrlType::StartRoutine, 0xff0a, crc32_data)?;

    Ok(())
}

