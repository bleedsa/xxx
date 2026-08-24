#![allow(non_snake_case)]

use criterion::{Criterion, criterion_group, criterion_main};
use std::ptr;
use xxx;

fn iota(x: usize) -> *mut u64 {
    unsafe {
        let P: *mut u64 = xxx::new(x).unwrap();

        for i in 0..x {
            ptr::write(P.add(i), i as u64);
        }

        P
    }
}

fn criterion_bench(c: &mut Criterion) {
    let mut B = |N, n, x, y: Result<*mut u64, String>| unsafe {
        let y = y.unwrap();
        let N2 = format!("{N} (std)");
        c.bench_function(N, |b| b.iter(|| xxx::cpy(y, x, n)));
        c.bench_function(&N2, |b| b.iter(|| std::ptr::copy(y, x, n)));
    };

    let small = 20;
    let med = 100_000;
    let big = 300_000;
    let huge = 500_000;

    unsafe {
        B("small copy", small, iota(small), xxx::new(small));
        B("med copy", med, iota(med), xxx::new(med));
        B("big copy", big, iota(big), xxx::new(big));
        B("huge copy", huge, iota(huge), xxx::new(huge));
    }

    unsafe {
        let x: *mut u64 = xxx::new(huge).unwrap();
        let y = iota(huge);
        c.bench_function("naive loop", |b| b.iter(|| {
            for i in 0..huge * 8 {
                *x.cast::<u8>().add(i) = *y.cast::<u8>().add(i);
            }
        }));
    }
}

criterion_group!(benches, criterion_bench);
criterion_main!(benches);
