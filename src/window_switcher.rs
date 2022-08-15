use std::{collections::HashMap, sync::RwLock};

use penrose::{common::geometry::Region, Xid};

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
