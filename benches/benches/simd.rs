// Copyright 2018-2023 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Generating SIMD / wide types

#![cfg_attr(feature = "simd_support", feature(portable_simd))]

use criterion::{criterion_group, criterion_main, Criterion};

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = simd
);
criterion_main!(benches);

#[cfg(not(feature = "simd_support"))]
pub fn simd(_: &mut Criterion) {}

#[cfg(feature = "simd_support")]
pub fn simd(c: &mut Criterion) {
    use rand::prelude::*;
    use rand_pcg::Pcg64Mcg;

    let mut g = c.benchmark_group("random_simd");

    g.bench_function("u128", |b| {
        let mut rng = Pcg64Mcg::from_rng(&mut rand::rng());
        b.iter(|| rng.random::<u128>());
    });

    g.bench_function("m128i", |b| {
        let mut rng = Pcg64Mcg::from_rng(&mut rand::rng());
        b.iter(|| rng.random::<core::arch::x86_64::__m128i>());
    });

    g.bench_function("m256i", |b| {
        let mut rng = Pcg64Mcg::from_rng(&mut rand::rng());
        b.iter(|| rng.random::<core::arch::x86_64::__m256i>());
    });

    g.bench_function("m512i", |b| {
        let mut rng = Pcg64Mcg::from_rng(&mut rand::rng());
        b.iter(|| rng.random::<core::arch::x86_64::__m512i>());
    });

    g.bench_function("u64x2", |b| {
        let mut rng = Pcg64Mcg::from_rng(&mut rand::rng());
        b.iter(|| rng.random::<core::simd::u64x2>());
    });

    g.bench_function("u32x4", |b| {
        let mut rng = Pcg64Mcg::from_rng(&mut rand::rng());
        b.iter(|| rng.random::<core::simd::u64x4>());
    });

    g.bench_function("u32x8", |b| {
        let mut rng = Pcg64Mcg::from_rng(&mut rand::rng());
        b.iter(|| rng.random::<core::simd::u8x32>());
    });

    g.bench_function("u16x8", |b| {
        let mut rng = Pcg64Mcg::from_rng(&mut rand::rng());
        b.iter(|| rng.random::<core::simd::u8x32>());
    });

    g.bench_function("u8x16", |b| {
        let mut rng = Pcg64Mcg::from_rng(&mut rand::rng());
        b.iter(|| rng.random::<core::simd::u8x32>());
    });
}
