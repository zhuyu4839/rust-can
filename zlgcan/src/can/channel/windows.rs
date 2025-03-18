use crate::can::common::{ZCanChlCfgInner, ZCanFdChlCfgInner};

#[repr(C)]
#[derive(Copy, Clone)]
pub union ZCanChlCfgUnion {
    pub(crate) can: ZCanChlCfgInner,
    pub(crate) canfd: ZCanFdChlCfgInner,
}
