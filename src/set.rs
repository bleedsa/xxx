use crate::{xmm_t, ymm_t};
use std::{
    hint::{likely, unlikely},
    mem::size_of,
    mem::{MaybeUninit, transmute},
    ptr,
};

pub unsafe fn set<X>(x: *mut X, b: u8, Z: usize) {
    /* everything into bytes */
    let mut x = x as *mut u8;
    let mut Z = Z * size_of::<X>();

    /* make various vectors out of b */
    let bu16: u16 =
        unsafe { ptr::read_unaligned([b, b].as_ptr() as *const u16) };
    let bu32: u32 =
        unsafe { ptr::read_unaligned([bu16, bu16].as_ptr() as *const u32) };
    let bu64: u64 =
        unsafe { ptr::read_unaligned([bu32, bu32].as_ptr() as *const u64) };
    let bxmm: xmm_t =
        unsafe { ptr::read_unaligned([bu64, bu64].as_ptr() as *const xmm_t) };
    let bymm: ymm_t =
        unsafe { ptr::read_unaligned([bxmm, bxmm].as_ptr() as *const ymm_t) };

    macro_rules! W {
        ($T:ty => $x:expr) => {{
            const ZOF: usize = size_of::<$T>();

            unsafe {
                let a = x.cast::<MaybeUninit<$T>>();
                ptr::write(a, transmute($x));
                x = x.add(ZOF);
            }

            Z -= ZOF;
        }};
    }

    if likely(Z >= 1) && !x.is_aligned_to(2) {
        W!(u8=>b);
    }

    if likely(Z >= 2) && !x.is_aligned_to(4) {
        W!(u16=>bu16);
    }

    if likely(Z >= 4) && !x.is_aligned_to(8) {
        W!(u32=>bu32);
    }

    if likely(Z >= 8) && !x.is_aligned_to(16) {
        W!(u64=>bu64);
    }

    if likely(Z >= 16) && !x.is_aligned_to(32) {
        W!(xmm_t=>bxmm);
    }

    while Z >= 32 {
        W!(ymm_t=>bymm);
    }

    if Z >= 16 {
        W!(xmm_t=>bxmm);
    }

    if Z >= 8 {
        W!(u64=>bu64);
    }

    while unlikely(Z >= 1) {
        W!(u8=>b);
    }
}

#[cfg(test)]
use crate::{free, new};

#[test]
fn Set_32() {
    unsafe {
        let n = 4;
        let x: *mut u8 = new(n).unwrap();
        let b = 100;

        set(x, b, n);

        for i in 0..n {
            assert_eq!(*x.add(i), b);
        }

        free(x, n);
    }
}

#[test]
fn Set_padded_mem() {
    unsafe {
        let n = 10;
        let x: *mut (u32, u64, u8) = new(n).unwrap();
        let b = 0;

        set(x, b, n);

        for i in 0..n {
            assert_eq!(*x.add(i), (0, 0, 0));
        }

        free(x, n);
    }
}
