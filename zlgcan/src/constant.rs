#![allow(dead_code)]

pub(crate) const LOAD_LIB_FAILED: &str = "ZLGCAN - could not open library!";
pub(crate) const STATUS_ONLINE: u32 = 2;
pub(crate) const STATUS_OFFLINE: u32 = 3;
pub(crate) const CLOCK: &str = "clock";
pub(crate) const INTERNAL_RESISTANCE: &str = "initenal_resistance";
pub(crate) const PROTOCOL: &str = "protocol";
pub(crate) const BAUD_RATE: &str = "baud_rate";
pub(crate) const CANFD_ABIT_BAUD_RATE: &str = "canfd_abit_baud_rate";
pub(crate) const CANFD_DBIT_BAUD_RATE: &str = "canfd_dbit_baud_rate";
pub(crate) const BAUD_RATE_CUSTOM: &str = "baud_rate_custom";
pub(crate) const CANFD_STANDARD: &str = "canfd_standard";
pub(crate) const TX_TIMEOUT: &str = "tx_timeout";
pub(crate) const AUTO_SEND: &str = "auto_send";
pub(crate) const AUTO_SEND_CANFD: &str = "auto_send_canfd";
pub(crate) const AUTO_SEND_PARAM: &str = "auto_send_param";
pub(crate) const CLEAR_AUTO_SEND: &str = "clear_auto_send";
pub(crate) const APPLY_AUTO_SEND: &str = "apply_auto_send";
pub(crate) const SET_SEND_MODE: &str = "set_send_mode";
pub(crate) const GET_DEVICE_AVAILABLE_TX_COUNT: &str = "get_device_available_tx_count/1";
pub(crate) const CLEAR_DELAY_SEND_QUEUE: &str = "clear_delay_send_queue";
pub(crate) const SET_DEVICE_RECV_MERGE: &str = "set_device_recv_merge";
pub(crate) const GET_DEVICE_RECV_MERGE: &str = "get_device_recv_merge/1";
pub(crate) const SET_CN: &str = "set_cn";
pub(crate) const SET_NAME: &str = "set_name";
pub(crate) const GET_CN: &str = "get_cn/1";
pub(crate) const GET_NAME : &str = "get_name/1 ";
pub(crate) const FILTER_MODE: &str = "filter_mode";
pub(crate) const FILTER_START: &str = "filter_start";
pub(crate) const FILTER_END: &str = "filter_end";
pub(crate) const FILTER_ACK: &str = "filter_ack";
pub(crate) const FILTER_CLEAR: &str = "filter_clear";
pub(crate) const SET_BUS_USAGE_ENABLE: &str = "set_bus_usage_enable";
pub(crate) const SET_BUS_USAGE_PERIOD: &str = "set_bus_usage_period";
pub(crate) const GET_BUS_USAGE: &str = "get_bus_usage/1";
pub(crate) const SET_TX_RETRY_POLICY: &str = "set_tx_retry_policy";
/// USBCAN-4E-U
#[inline]
pub(crate) fn channel_bitrate(channel: u8) -> String {
    format!("info/channel/channel_{}/baud_rate", channel)
}
/// USBCAN-4E-U
#[inline]
pub(crate) fn channel_work_mode(channel: u8) -> String {
    format!("info/channel/channel_{}/work_mode", channel)
}
/// USBCAN-8E-U and USBCAN-4E-U
#[inline]
pub(crate) fn channel_redirect(channel: u8) -> String {
    format!("info/channel/channel_{}/redirect", channel)
}
/// USBCAN-4E-U
#[inline]
pub(crate) fn channel_whitelisting(channel: u8) -> String {
    format!("info/channel/channel_{}/whitelisting", channel)
}
/// USBCAN-4E-U
#[inline]
pub(crate) fn channel_auto_trans(channel: u8) -> String {
    format!("info/channel/channel_{}/autotxobj", channel)
}
