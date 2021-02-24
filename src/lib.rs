#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

include!(concat!(env!("OUT_DIR"), "/regex.rs"));

pub fn io_res(r: std::os::raw::c_int) -> std::io::Result<()> {
    if r == -1 {
        Err(std::io::Error::last_os_error())
    } else { Ok(()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn get_version() {
        use std::fs::File;
        let c0 = if let Ok(c0) = File::open("/dev/snd/controlC0") { c0 } else { return };
        use std::os::unix::io::AsRawFd;
        let fd = c0.as_raw_fd();
        let mut ver = 0;
        io_res(unsafe { SNDRV_CTL_IOCTL_PVERSION(fd, &mut ver) }).unwrap();
        println!("Protocol version is {}.{}.{}", ver >> 16, (ver >> 8) & 0xff, ver & 0xff);
        assert_eq!(ver >> 16, 2);
    }
}
