extern crate discord_rpc_sys as sys;
extern crate libc;

#[macro_use]
extern crate lazy_static;

pub mod events;

#[cfg(all(windows, target_pointer_width = "64"))]
mod rpc;

#[cfg(not(all(windows, target_pointer_width = "64")))]
mod rpc {
    //TODO: implement this
}

pub use rpc::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
