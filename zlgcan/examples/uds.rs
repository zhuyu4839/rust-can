mod utils;

use std::time::Duration;
use hex_literal::hex;
use isotp_rs::can::Address;
use ecu_uds::service::{CommunicationCtrlType, CommunicationType, DataIdentifier, DTCSettingType, ECUResetType, RoutineCtrlType, SessionType};
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

    client.session_ctrl(CHANNEL, SessionType::Extended, false, false)?;

    client.unlock_security_access(CHANNEL, 1, vec![], hex!("01 41 32 30").to_vec())?;

    let vin = hex!("30 30 30 30 30 30 30 30 30 30 30 30 30 30 30 30 30").to_vec();
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

    client.session_ctrl(CHANNEL, SessionType::Extended, false, false)?;

    client.communication_control(
        0,
        CommunicationCtrlType::EnableRxAndTx,
        CommunicationType::NormalCommunicationMessages,
        None,
        false,
        false
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

    client.session_ctrl(CHANNEL, SessionType::Extended, false, false)?;
    let result = client.security_access(CHANNEL, 1, vec![])?;
    println!("security access response: {:?}", result);
    let param = uds_security_algo(1, result, hex!("01 41 32 30").to_vec())?;
    let result = client.security_access(CHANNEL, 2, param)?;
    println!("security access response: {:?}", result);

    let result = client.io_control(
        0,
        DataIdentifier::from(0x4101),
        ecu_uds::service::IOCtrlParameter::ShortTermAdjustment,
        vec![0x00, 0x40],
        vec![0xff, 0xff],
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

    client.session_ctrl(CHANNEL, SessionType::Extended, false, true)?;

    client.routine_control(CHANNEL, RoutineCtrlType::StartRoutine, 0xdf01, vec![])?;
    client.control_dtc_setting(CHANNEL, DTCSettingType::Off, vec![], false)?;
    client.communication_control(CHANNEL, CommunicationCtrlType::DisableRxAndTx, CommunicationType::NormalCommunicationMessages, None, false, false)?;

    client.session_ctrl(CHANNEL, SessionType::Programming, false, false)?;

    let result = client.security_access(CHANNEL, 0x11, vec![])?;
    let param = uds_security_algo(0x11, result, hex!("01 41 32 30").to_vec())?;
    client.security_access(CHANNEL, 0x12, param)?;

    client.write_data_by_identifier(CHANNEL, DataIdentifier::ProgrammingDate, vec![0x20, 0x24, 0x08, 0x21])?;

    uds_flash_file(
        "/home/manjaro/Workspace/RustProjects/ecu-proto-rs/resources/1-BMS_0001_V001.hex",
        &mut client,
        CHANNEL,
        false,
    )?;

    // std::thread::sleep(Duration::from_millis(100));

    uds_flash_file(
        "/home/manjaro/Workspace/RustProjects/ecu-proto-rs/resources/2-BMS_DVC7E4000001_S1000041_A01_0001.hex",
        &mut client,
        CHANNEL,
        true
    )?;

    client.routine_control(CHANNEL, RoutineCtrlType::StartRoutine, 0xff01, vec![])?;
    client.ecu_reset(CHANNEL, ECUResetType::HardReset, false, false)?;

    std::thread::sleep(Duration::from_secs(5));

    client.session_ctrl(CHANNEL, SessionType::Extended, false, true)?;
    client.communication_control(CHANNEL, CommunicationCtrlType::EnableRxAndTx, CommunicationType::NormalCommunicationMessages, None, false, false)?;
    client.control_dtc_setting(CHANNEL, DTCSettingType::On, vec![], false)?;

    device.stop();

    Ok(())
}

