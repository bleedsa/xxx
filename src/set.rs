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

    /* make a big array of b we can copy out of */
    let B = [b; 32];
    macro_rules! p {
        ($t:ty) => {{ unsafe { ptr::read_unaligned(B.as_ptr() as *const $t) } }};
    }

    /* make various vectors out of b */
    let bu16 = p!(u16);
    let bu32 = p!(u32);
    let bu64 = p!(u64);
    let bxmm = p!(xmm_t);
    let bymm = p!(ymm_t);

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

    while Z >= 64 {
        W!(ymm_t=>bymm);
        W!(ymm_t=>bymm);
    }

    if Z >= 32 {
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

#[test]
fn Set_struct_mem() {
    #[derive(Debug, PartialEq)]
    struct A(u8, u16);

    unsafe {
        let n = 20;
        let x: *mut A = new(n).unwrap();
        let b = 0b11111111;

        set(x, b, n);

        for i in 0..n {
            assert_eq!(*x.add(i), A(0b11111111, 0b1111111111111111));
        }

        free(x, n);
    }
}
