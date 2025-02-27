mod utils;

use std::time::Duration;
use docan::Client;
use iso14229_1::{CommunicationCtrlType, CommunicationType, DataIdentifier, DTCSettingType, ECUResetType, RoutineCtrlType, SessionType, IOCtrlParameter};
use rs_can::isotp::{Address, AddressType};
use crate::utils::{uds_flash_file, algo::uds_security_algo, CHANNEL};


#[test]
#[ignore]
fn test_write_did() -> anyhow::Result<()> {
    let (mut device, mut client) = utils::init_client()?;

    client.update_address(CHANNEL, Address {
        tx_id: 0x7E4,
        rx_id: 0x7EC,
        fid: 0x7DF,
    })?;
    client.update_security_algo(CHANNEL, uds_security_algo)?;

    client.session_ctrl(CHANNEL, SessionType::Extended, false, AddressType::Physical)?;

    client.unlock_security_access(CHANNEL, 1, vec![], hex::decode("01413230")?)?;

    let vin = hex::decode("3030303030303030303030303030303030")?;
    let result = client.write_data_by_identifier(CHANNEL, DataIdentifier::VIN, vin)?;
    println!("write DID response: {:?}", result);

    device.stop();

    Ok(())
}

#[test]
#[ignore]
fn test_read_did() -> anyhow::Result<()> {
    let (mut device, mut client) = utils::init_client()?;

    client.update_address(CHANNEL, Address {
        tx_id: 0x7E4,
        rx_id: 0x7EC,
        fid: 0x7DF,
    })?;

    client.add_data_identifier(CHANNEL, DataIdentifier::VIN, 17)?;
    client.add_data_identifier(CHANNEL, DataIdentifier::BootSoftwareFingerprint, 16)?;
    let result = client.read_data_by_identifier(CHANNEL, DataIdentifier::VIN, vec![])?;
    println!("read DID response: {:?}", result);

    let result = client.read_data_by_identifier(CHANNEL, DataIdentifier::VIN, vec![
        DataIdentifier::BootSoftwareFingerprint
    ])?;
    println!("read DID response: {:?}", result);

    device.stop();

    Ok(())
}

#[test]
#[ignore]
fn test_communication_control() -> anyhow::Result<()> {
    let (mut device, mut client) = utils::init_client()?;

    client.session_ctrl(CHANNEL, SessionType::Extended, false, AddressType::Physical)?;

    client.communication_control(
        0,
        CommunicationCtrlType::EnableRxAndTx,
        CommunicationType::NormalCommunicationMessages,
        None,
        false,
        AddressType::Physical
    )?;
    device.stop();

    Ok(())
}

#[test]
#[ignore]
fn test_io_control() -> anyhow::Result<()> {
    let (mut device, mut client) = utils::init_client()?;

    client.update_address(CHANNEL, Address {
        tx_id: 0x7E6,
        rx_id: 0x7EE,
        fid: 0x7DF,
    })?;
    client.update_security_algo(CHANNEL, uds_security_algo)?;
    let did = DataIdentifier::from(0x4101);
    client.add_data_identifier(0, did, 2)?;

    client.session_ctrl(CHANNEL, SessionType::Extended, false, AddressType::Physical)?;
    let result = client.security_access(CHANNEL, 1, vec![])?;
    println!("security access response: {:?}", result);
    let param = uds_security_algo(1, result, hex::decode("01413230")?)?;
    let result = client.security_access(CHANNEL, 2, param.unwrap())?;
    println!("security access response: {:?}", result);

    let result = client.io_control(
        0,
        did,
        IOCtrlParameter::ShortTermAdjustment,
        vec![0xff, 0xff],
        vec![0x00, 0xff],
    )?;
    print!("io control response: {:?}", result);

    device.stop();

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let (mut device, mut client) = utils::init_client()?;

    client.update_address(CHANNEL, Address {
        tx_id: 0x7E4,
        rx_id: 0x7EC,
        fid: 0x7DF,
    })?;
    client.update_security_algo(CHANNEL, uds_security_algo)?;
    client.add_data_identifier(CHANNEL, DataIdentifier::ProgrammingDate, 4)?;

    client.session_ctrl(CHANNEL, SessionType::Extended, false, AddressType::Functional)?;

    client.routine_control(CHANNEL, RoutineCtrlType::StartRoutine, 0xdf01, vec![])?;
    client.control_dtc_setting(CHANNEL, DTCSettingType::Off, vec![], false)?;
    client.communication_control(CHANNEL, CommunicationCtrlType::DisableRxAndTx, CommunicationType::NormalCommunicationMessages, None, false, AddressType::Physical)?;

    client.session_ctrl(CHANNEL, SessionType::Programming, false, AddressType::Physical)?;

    let result = client.security_access(CHANNEL, 0x11, vec![])?;
    let param = uds_security_algo(0x11, result, hex::decode("01413230")?)?;
    client.security_access(CHANNEL, 0x12, param.unwrap())?;

    client.write_data_by_identifier(CHANNEL, DataIdentifier::ProgrammingDate, vec![0x20, 0x24, 0x08, 0x21])?;

    uds_flash_file(
        "/home/manjaro/Downloads/davinci/BMS/1-BMS_0001_V001.hex",
        &mut client,
        CHANNEL,
        false,
    )?;

    // std::thread::sleep(Duration::from_millis(100));

    uds_flash_file(
        "/home/manjaro/Downloads/davinci/BMS/S0000043.hex",
        &mut client,
        CHANNEL,
        true
    )?;

    client.routine_control(CHANNEL, RoutineCtrlType::StartRoutine, 0xff01, vec![])?;
    client.ecu_reset(CHANNEL, ECUResetType::HardReset, false, AddressType::Physical)?;

    std::thread::sleep(Duration::from_secs(5));

    client.session_ctrl(CHANNEL, SessionType::Extended, false, AddressType::Functional)?;
    client.communication_control(CHANNEL, CommunicationCtrlType::EnableRxAndTx, CommunicationType::NormalCommunicationMessages, None, false, AddressType::Physical)?;
    client.control_dtc_setting(CHANNEL, DTCSettingType::On, vec![], false)?;

    device.stop();

    Ok(())
}

