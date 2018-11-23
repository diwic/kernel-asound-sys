#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[macro_use]
extern crate nix;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

include!(concat!(env!("OUT_DIR"), "/regex.rs"));

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn get_version() {
        use std::fs::File;
        let c0 = File::open("/dev/snd/controlC0").unwrap();
        use std::os::unix::io::AsRawFd;
        let fd = c0.as_raw_fd();
        let mut ver = 0;
        unsafe { SNDRV_CTL_IOCTL_PVERSION(fd, &mut ver).unwrap(); }
        println!("Protocol version is {}.{}.{}", ver >> 16, (ver >> 8) & 0xff, ver & 0xff);
    }
}
