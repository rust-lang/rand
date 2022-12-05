// Copyright 2018-2022 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use criterion::{criterion_group, criterion_main, Criterion};
use criterion_cycles_per_byte::CyclesPerByte;
use rand::prelude::*;

// We force use of 32-bit RNG since seq code is optimised for use with 32-bit
// generators on all platforms.
use rand_chacha::ChaCha20Rng as CryptoRng;
use rand_pcg::Pcg32 as SmallRng;

criterion_group!(
name = benches;
config = Criterion::default().with_measurement(CyclesPerByte);
targets = bench
);
criterion_main!(benches);

pub fn bench(c: &mut Criterion<CyclesPerByte>) {
    for length in [1, 2, 3, 10, 100, 1000] {
        c.bench_function(format!("choose_from_{length}_small").as_str(), |b| {
            let mut rng = SmallRng::seed_from_u64(123);
            b.iter(|| choose(length, &mut rng))
        });

        c.bench_function(format!("choose_stable_from_{length}_small").as_str(), |b| {
            let mut rng = SmallRng::seed_from_u64(123);
            b.iter(|| choose_stable(length, &mut rng))
        });

        c.bench_function(
            format!("choose_unhinted_from_{length}_small").as_str(),
            |b| {
                let mut rng = SmallRng::seed_from_u64(123);
                b.iter(|| choose_unhinted(length, &mut rng))
            },
        );

        c.bench_function(
            format!("choose_windowed_from_{length}_small").as_str(),
            |b| {
                let mut rng = SmallRng::seed_from_u64(123);
                b.iter(|| choose_windowed(length, 7, &mut rng))
            },
        );

        c.bench_function(format!("choose_from_{length}_crypto").as_str(), |b| {
            let mut rng = CryptoRng::seed_from_u64(123);
            b.iter(|| choose(length, &mut rng))
        });

        c.bench_function(
            format!("choose_stable_from_{length}_crypto").as_str(),
            |b| {
                let mut rng = CryptoRng::seed_from_u64(123);
                b.iter(|| choose_stable(length, &mut rng))
            },
        );

        c.bench_function(
            format!("choose_unhinted_from_{length}_crypto").as_str(),
            |b| {
                let mut rng = CryptoRng::seed_from_u64(123);
                b.iter(|| choose_unhinted(length, &mut rng))
            },
        );

        c.bench_function(
            format!("choose_windowed_from_{length}_crypto").as_str(),
            |b| {
                let mut rng = CryptoRng::seed_from_u64(123);
                b.iter(|| choose_windowed(length, 7, &mut rng))
            },
        );
    }
}

fn choose<R: Rng>(max: usize, rng: &mut R) -> Option<usize> {
    let iterator = 0..max;
    iterator.choose(rng)
}

fn choose_stable<R: Rng>(max: usize, rng: &mut R) -> Option<usize> {
    let iterator = 0..max;
    iterator.choose_stable(rng)
}

fn choose_unhinted<R: Rng>(max: usize, rng: &mut R) -> Option<usize> {
    let iterator = UnhintedIterator { iter: (0..max) };
    iterator.choose(rng)
}

fn choose_windowed<R: Rng>(max: usize, window_size: usize, rng: &mut R) -> Option<usize> {
    let iterator = WindowHintedIterator {
        iter: (0..max),
        window_size,
    };
    iterator.choose(rng)
}

#[derive(Clone)]
struct UnhintedIterator<I: Iterator + Clone> {
    iter: I,
}
impl<I: Iterator + Clone> Iterator for UnhintedIterator<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

#[derive(Clone)]
struct WindowHintedIterator<I: ExactSizeIterator + Iterator + Clone> {
    iter: I,
    window_size: usize,
}
impl<I: ExactSizeIterator + Iterator + Clone> Iterator for WindowHintedIterator<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (core::cmp::min(self.iter.len(), self.window_size), None)
    }
}
