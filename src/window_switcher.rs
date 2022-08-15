use std::{collections::HashMap, sync::RwLock};

use penrose::{core::data_types::Region, Xid};

use lazy_static::lazy_static;

lazy_static! {
    pub static ref CLIENT_REGIONS: RwLock<HashMap<Xid, Region>> = RwLock::new(HashMap::new());
}

pub enum SwitchDirection {
    Up,
    Down,
    Left,
    Right,
}
