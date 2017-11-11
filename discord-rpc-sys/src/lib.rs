extern crate libc;

#[cfg(all(windows, target_pointer_width = "64"))]
mod imp;

#[cfg(not(all(windows, target_pointer_width = "64")))]
mod imp {
    //TODO: implement this
}

pub use imp::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

