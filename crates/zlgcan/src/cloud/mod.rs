use std::ffi::{c_char, c_int, c_uchar, c_ushort};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ZCloudServerInfo {
    pub http_url: *const c_char,
    pub http_port: u16,
    pub mqtt_url: *const c_char,
    pub mqtt_port: u16,
}

#[allow(non_snake_case)]
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct ZCloudChlInfo {
    pub enable: c_uchar,
    pub type_: c_uchar,
    pub isUpload: c_uchar,
    pub isDownload: c_uchar,
}

#[allow(non_snake_case)]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ZCloudDeviceInfo {
    pub devIndex: c_int,
    pub type_: [c_char; 64usize],
    pub id: [c_char; 64usize],
    pub name: [c_char; 64usize],
    pub owner: [c_char; 64usize],
    pub model: [c_char; 64usize],
    pub fwVer: [c_char; 16usize],
    pub hwVer: [c_char; 16usize],
    pub serial: [c_char; 64usize],
    pub status: c_int,
    pub bGpsUpload: c_uchar,
    pub channelCnt: c_uchar,
    pub channels: [ZCloudChlInfo; 16usize],
}

impl Default for ZCloudDeviceInfo {
    fn default() -> Self {
        Self {
            devIndex: Default::default(),
            type_: [Default::default(); 64usize],
            id: [Default::default(); 64usize],
            name: [Default::default(); 64usize],
            owner: [Default::default(); 64usize],
            model: [Default::default(); 64usize],
            fwVer: Default::default(),
            hwVer: Default::default(),
            serial: [Default::default(); 64usize],
            status: Default::default(),
            bGpsUpload: Default::default(),
            channelCnt: Default::default(),
            channels: [Default::default(); 16usize],
        }
    }
}

#[allow(non_snake_case)]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ZCloudUserData {
    pub username: [c_char; 64usize],
    pub mobile: [c_char; 64usize],
    pub dllVer: [c_char; 16usize],
    pub devCnt: usize,
    pub devices: [ZCloudDeviceInfo; 100usize],
}

impl Default for ZCloudUserData {
    fn default() -> Self {
        Self {
            username: [Default::default(); 64usize],
            mobile: [Default::default(); 64usize],
            dllVer: Default::default(),
            devCnt: Default::default(),
            devices: [Default::default(); 100usize],
        }
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct ZCloudGpsFrame {
    pub latitude: f32,
    pub longitude: f32,
    pub speed: f32,
    pub tm: ZCloudGpsTime,
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct ZCloudGpsTime {
    pub year: c_ushort,
    pub mon: c_ushort,
    pub day: c_ushort,
    pub hour: c_ushort,
    pub min: c_ushort,
    pub sec: c_ushort,
}
