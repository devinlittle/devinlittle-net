use std::sync::OnceLock;

use crate::structs::Auth;
use arc_swap::ArcSwap;

pub mod structs;

static AUTH_STATE: OnceLock<ArcSwap<Auth>> = OnceLock::new();

pub fn auth_state() -> &'static ArcSwap<Auth> {
    AUTH_STATE.get_or_init(|| ArcSwap::from_pointee(Auth::default()))
}

pub fn init_prog() {}

// INFO: random stuff
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

pub fn print_from_dln_core() {
    println!("this was printed from the dln_core");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
