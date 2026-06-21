use std::ptr::null_mut;

#[link(name = "ntdll")]
unsafe extern "system" {
    fn RtlGetNtVersionNumbers(major: *mut u32, minor: *mut u32, build: *mut u32);
}

#[inline]
pub(crate) fn get_nt_build() -> u32 {
    let mut build = 0;
    unsafe { RtlGetNtVersionNumbers(null_mut(), null_mut(), &mut build) };
    build &= !0xF0000000;
    build
}
