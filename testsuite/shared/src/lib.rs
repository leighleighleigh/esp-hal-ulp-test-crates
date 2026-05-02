#![no_std]
#![no_main]

// Where is the counter placed in memory
pub const COUNTER_ADDRESS: usize = 0x5000_1000;

// Where is the LP/ULP mode setting
pub const MODE_ADDRESS: usize = 0x5000_1004;

#[inline]
pub fn reg_read(addr: usize) -> u32 {
    unsafe {
        let counter = addr as *mut u32;
        counter.read_volatile()
    }
}

#[inline]
pub fn reg_write(addr: usize, val: u32) {
    unsafe {
        let counter = addr as *mut u32;
        counter.write_volatile(val);
    }
}

// pub fn add(left: u64, right: u64) -> u64 {
//     left + right
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
