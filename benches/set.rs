#![allow(non_snake_case)]

use criterion::{Criterion, criterion_group, criterion_main};
use xxx;

fn criterion_bench(c: &mut Criterion) {
    let mut B = |N, n, x: Result<*mut usize, String>| unsafe {
        let x = x.unwrap();
        let N2 = format!("{N} (std)");
        c.bench_function(N, |b| b.iter(|| xxx::set(x, 0, n)));
        c.bench_function(&N2, |b| {
            b.iter(|| libc::memset(x as *mut libc::c_void, 0, n))
        });
    };

    let small = 20;
    let med = 100_000;
    let big = 300_000;
    let huge = 500_000;

    unsafe {
        B("small set", small, xxx::new(small));
        B("med set", med, xxx::new(med));
        B("big set", big, xxx::new(big));
        B("huge set", huge, xxx::new(huge));
    }
}

criterion_group!(benches, criterion_bench);
criterion_main!(benches);
