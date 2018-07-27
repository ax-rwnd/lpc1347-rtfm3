#![allow(dead_code)]

extern crate core;
extern crate lpc1347;

const IAP_LOCATION: u32 = 0x1fff1ff1;

fn read_uid() -> [u32; 4] {
    let command: [u32; 5] = [57, 0, 0, 0, 0];
    let result: [u32; 4] = [0, 0, 0, 0];

    let ptr = IAP_LOCATION as *const ();
    let iap: extern "C" fn(*const u32, *const u32) = unsafe { core::mem::transmute(ptr) };
    (iap)(&command as *const u32, &result as *const u32);

    return result;
}
