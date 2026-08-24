use crate::{xmm_t, ymm_t};
use std::{
    hint::{likely, unlikely},
    mem::MaybeUninit,
    ptr,
};

#[cfg(test)]
use crate::{free, new};

/**
 * `memcpy(dst(x), src(y), size(Z));`
 *
 * vectorized memcpy. copies `x` <- `y` over `Z` elems.
 */
pub unsafe fn cpy<X>(x: *mut X, y: *mut X, Z: usize) {
    /* everything into bytes */
    let mut x = x as *mut u8;
    let mut y = y as *mut u8;
    let mut Z = Z * size_of::<X>();

    /* copy a pointer from y into x with type ($T) for the size of the move. */
    macro_rules! W {
        ($T:ty) => {{
            /* size of $T */
            const ZOF: usize = size_of::<$T>();

            unsafe {
                /* convert our ptrs
                 * mark as MaybeUninit because some things are padded
                 * with garbage data (ie structs/enums/Option). */
                let a = x.cast::<MaybeUninit<$T>>();
                let b = y.cast::<MaybeUninit<$T>>();

                /* grab the data from y */
                let d = ptr::read_unaligned(b);

                /* perform the write */
                ptr::write(a, d);

                /* inc the ptrs */
                x = x.add(ZOF);
                y = y.add(ZOF);
            }

            /* adjust the size */
            Z -= ZOF;
        }};
    }

    /* perform some initial writes so that we can perform aligned writes.
     * thanks barrow! */
    if likely(Z >= 1) && !x.is_aligned_to(2) {
        W!(u8);
    }

    if likely(Z >= 2) && !x.is_aligned_to(4) {
        W!(u16);
    }

    if likely(Z >= 4) && !x.is_aligned_to(8) {
        W!(u32);
    }

    if likely(Z >= 8) && !x.is_aligned_to(16) {
        W!(u64);
    }

    if likely(Z >= 16) && !x.is_aligned_to(32) {
        W!(xmm_t);
    }

    /* if we have a shit ton of bytes, we perform a bunch of writes in a row.
     * takes less iters. */
    while likely(Z >= 128) {
        W!(ymm_t);
        W!(ymm_t);
        W!(ymm_t);
        W!(ymm_t);
    }

    if Z >= 64 {
        W!(ymm_t);
        W!(ymm_t);
    }

    /* now we start doing regular reg writes */
    if Z >= 32 {
        W!(ymm_t);
    }

    if Z >= 16 {
        W!(xmm_t);
    }

    if Z >= 8 {
        W!(u64);
    }

    /* skip u32&u16, just copy the rest of the bytes one by one. */
    while unlikely(Z >= 1) {
        W!(u8);
    }
}

#[test]
fn Cpy_32() {
    unsafe {
        let x = new(8).unwrap();
        let y = new(8).unwrap();

        *x = 0u64;
        *x.add(1) = 1;
        *x.add(2) = 2;
        *x.add(3) = 3;
        *x.add(4) = 4;
        *x.add(5) = 5;
        *x.add(6) = 6;
        *x.add(7) = 7;

        cpy(y, x, 8);

        assert_eq!(*x, *y);
        assert_eq!(*x.add(1), *y.add(1));
        assert_eq!(*x.add(2), *y.add(2));
        assert_eq!(*x.add(3), *y.add(3));
        assert_eq!(*x.add(4), *y.add(4));
        assert_eq!(*x.add(5), *y.add(5));
        assert_eq!(*x.add(6), *y.add(6));
        assert_eq!(*x.add(7), *y.add(7));

        free(x, 8);
        free(y, 8);
    }
}

#[test]
fn Cpy_16() {
    unsafe {
        let x = new(4).unwrap();
        let y = new(4).unwrap();

        *x = 0u64;
        *x.add(1) = 1;
        *x.add(2) = 2;
        *x.add(3) = 3;

        cpy(y, x, 4);

        assert_eq!(*x, *y);
        assert_eq!(*x.add(1), *y.add(1));
        assert_eq!(*x.add(2), *y.add(2));
        assert_eq!(*x.add(3), *y.add(3));

        free(x, 4);
        free(y, 4);
    }
}

#[test]
fn Cpy_8() {
    unsafe {
        let x: *mut u32 = new(2).unwrap();
        let y: *mut u32 = new(2).unwrap();

        *x = 0u32;
        *x.add(1) = 1;

        cpy(y, x, 2);

        assert_eq!(*x, *y);
        assert_eq!(*x.add(1), *y.add(1));

        free(x, 2);
        free(y, 2);
    }
}

#[test]
fn Cpy_4() {
    unsafe {
        let x = new(2).unwrap();
        let y = new(2).unwrap();

        *x = 0u16;
        *x.add(1) = 1;

        cpy(y, x, 2);

        assert_eq!(*x, *y);
        assert_eq!(*x.add(1), *y.add(1));

        free(x, 2);
        free(y, 2);
    }
}

#[test]
fn Cpy_2() {
    unsafe {
        let x = new(2).unwrap();
        let y = new(2).unwrap();

        *x = 0u8;
        *x.add(1) = 1;

        cpy(y, x, 2);

        assert_eq!(*x, *y);
        assert_eq!(*x.add(1), *y.add(1));

        free(x, 2);
        free(y, 2);
    }
}

#[test]
fn Cpy_1() {
    unsafe {
        let x = new(1).unwrap();
        let y = new(1).unwrap();

        *x = 0u8;

        cpy(y, x, 1);

        assert_eq!(*x, *y);

        free(x, 1);
        free(y, 1);
    }
}

#[test]
fn Cpy_16_8() {
    unsafe {
        let x = new(3).unwrap();
        let y = new(3).unwrap();

        *x = 0u64;
        *x.add(1) = 1;
        *x.add(2) = 2;

        cpy(y, x, 3);

        assert_eq!(*x, *y);
        assert_eq!(*x.add(1), *y.add(1));
        assert_eq!(*x.add(2), *y.add(2));

        free(x, 3);
        free(y, 3);
    }
}

#[test]
fn Cpy_16_4() {
    unsafe {
        let x = new(5).unwrap();
        let y = new(5).unwrap();

        *x = 0u32;
        *x.add(1) = 1;
        *x.add(2) = 2;
        *x.add(3) = 3;
        *x.add(4) = 4;

        cpy(y, x, 5);

        assert_eq!(*x, *y);
        assert_eq!(*x.add(1), *y.add(1));
        assert_eq!(*x.add(2), *y.add(2));
        assert_eq!(*x.add(3), *y.add(3));
        assert_eq!(*x.add(4), *y.add(4));

        free(x, 5);
        free(y, 5);
    }
}

#[test]
fn Cpy_padded_mem() {
    unsafe {
        let x = new(3).unwrap();
        let y = new(3).unwrap();

        *x = (None, Some(0u32));
        *x.add(1) = (Some(1u8), None);
        *x.add(2) = (Some(2), Some(2));

        cpy(y, x, 3);

        assert_eq!(*x, *y);
        assert_eq!(*x.add(1), *y.add(1));
        assert_eq!(*x.add(2), *y.add(2));

        free(x, 3);
        free(y, 3);
    }
}

#[test]
fn Cpy_struct_repr_c() {
    #[repr(C)]
    #[derive(Copy, Clone, PartialEq, Debug)]
    struct A(u8, u32);

    unsafe {
        let x = new(2).unwrap();
        let y = new(2).unwrap();

        *x = A(0, 0);
        *x.add(1) = A(1, 1);

        cpy(y, x, 2);

        assert_eq!(*x, *y);
        assert_eq!(*x.add(1), *y.add(1));

        free(x, 2);
        free(y, 2);
    }
}
