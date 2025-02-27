#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct CanFilter {
    pub can_id: u32,
    pub can_mask: u32,
}

impl From<(u32, u32)> for CanFilter {
    fn from((id, mask): (u32, u32)) -> Self {
        Self {
            can_id: id,
            can_mask: mask,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ChannelConfig {
    pub bitrate: u32,
    pub dbitrate: u32,
}
