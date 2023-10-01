use std::{
    ops::Sub,
    time::{SystemTime, UNIX_EPOCH},
};

pub fn time_in_seconds() -> u64 {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => panic!("SystemTime before UNIX_EPOCH!"),
    }
}
