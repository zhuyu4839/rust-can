#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct CanFilter {
    pub can_id: u32,
    pub can_mask: u32,
    pub extended: bool
}

#[derive(Debug, Copy, Clone)]
pub struct ChannelConfig {
    pub bitrate: u32,
    pub dbitrate: u32,
}
