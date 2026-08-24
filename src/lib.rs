#![feature(likely_unlikely)]
#![feature(repr_simd)]
#![feature(pointer_is_aligned_to)]
#![allow(internal_features)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use std::{
    alloc::{self, Layout},
    hint::unlikely,
    ptr,
};

pub mod cpy;
pub mod set;

pub use crate::{cpy::*, set::*};

pub(crate) type R<X> = Result<X, String>;

/** xmm register repr */
#[repr(simd)]
#[derive(Copy, Clone)]
pub(crate) struct xmm_t([u8; 16]);

/** ymm register repr */
#[repr(simd)]
#[derive(Copy, Clone)]
pub(crate) struct ymm_t([u8; 32]);

/** grab a new chunk of memory for an array of `Z` elements of type `X`. */
#[inline(always)]
pub unsafe fn new<X>(Z: usize) -> R<*mut X> {
    if unlikely(Z == 0) {
        return Err("cannot alloc(0).".to_string());
    }

    /* create an array */
    let lay = Layout::from_size_align(Z * size_of::<X>(), align_of::<X>())
        .map_err(|e| format!("{e}"))?;
    let ptr = unsafe { alloc::alloc(lay) as *mut X };

    /* check for oom */
    if unlikely(ptr as *const X == ptr::null()) {
        return Err("alloc::alloc() returned null.".to_string());
    }

    Ok(ptr)
}

#[test]
fn Alloc() {
    unsafe {
        let ptr: *mut usize = new(3).unwrap();
        *ptr = 1;
        *ptr.add(1) = 2;
        *ptr.add(2) = 3;

        assert_eq!(*ptr, 1);
        assert_eq!(*ptr.add(1), 2);
        assert_eq!(*ptr.add(2), 3);

        free(ptr, 3);
    }
}

/** you get it. */
#[inline(always)]
pub unsafe fn free<X>(x: *mut X, Z: usize) {
    let lay = Layout::from_size_align(size_of::<X>() * Z, align_of::<X>())
        .map_err(|e| format!("{e}"))
        .expect("invalid layout in free");
    unsafe {
        alloc::dealloc(x as *mut u8, lay);
    }
}

#[test]
fn current_alignment() {
    let test = |a: usize, b: usize| assert_eq!(1 << a.trailing_zeros(), b);
    test(128 + 1, 1);
    test(128 + 2, 2);
    test(128 + 4, 4);
    test(128 + 8, 8);
    test(128 + 16, 16);
}
