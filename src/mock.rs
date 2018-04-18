// Copyright 2018 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// https://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Mock random number generator

use rand_core::{RngCore, Error, impls};

/// A simple implementation of `RngCore` for testing purposes.
/// 
/// This generates an arithmetic sequence (i.e. adds a constant each step)
/// over a `u64` number, using wrapping arithmetic. If the increment is 0
/// the generator yields a constant.
/// 
/// ```rust
/// use rand::Rng;
/// use rand::mock::StepRng;
/// 
/// let mut my_rng = StepRng::new(2, 1);
/// let sample: [u64; 3] = my_rng.gen();
/// assert_eq!(sample, [2, 3, 4]);
/// ```
#[derive(Debug, Clone)]
pub struct StepRng {
    v: u64,
    a: u64,
}

impl StepRng {
    /// Create a `StepRng`, yielding an arithmetic sequence starting with
    /// `initial` and incremented by `increment` each time.
    pub fn new(initial: u64, increment: u64) -> Self {
        StepRng { v: initial, a: increment }
    }
}

impl RngCore for StepRng {
    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    fn next_u64(&mut self) -> u64 {
        let result = self.v;
        self.v = self.v.wrapping_add(self.a);
        result
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        impls::fill_bytes_via_next(self, dest);
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        Ok(self.fill_bytes(dest))
    }

    fn bytes_per_round(&self) -> usize { 8 }
}
