// Copyright 2018-2023 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! The PCG random number generators.
//!
//! This is a native Rust implementation of a small selection of PCG generators.
//! The primary goal of this crate is simple, minimal, well-tested code; in
//! other words it is explicitly not a goal to re-implement all of PCG.
//!
//! This crate provides:
//!
//! -   `Pcg32` aka `Lcg64Xsh32`, officially known as `pcg32`, a general
//!     purpose RNG. This is a good choice on both 32-bit and 64-bit CPUs
//!     (for 32-bit output).
//! -   `Pcg64` aka `Lcg128Xsl64`, officially known as `pcg64`, a general
//!     purpose RNG. This is a good choice on 64-bit CPUs.
//! -   `Pcg64Mcg` aka `Mcg128Xsl64`, officially known as `pcg64_fast`,
//!     a general purpose RNG using 128-bit multiplications. This has poor
//!     performance on 32-bit CPUs but is a good choice on 64-bit CPUs for
//!     both 32-bit and 64-bit output.
//!
//! Both of these use 16 bytes of state and 128-bit seeds, and are considered
//! value-stable (i.e. any change affecting the output given a fixed seed would
//! be considered a breaking change to the crate).
//!
//! # Example
//!
//! To initialize a generator, use the [`SeedableRng`][rand_core::SeedableRng] trait:
//!
//! ```
//! use rand_core::{SeedableRng, RngCore};
//! use rand_pcg::Pcg64Mcg;
//!
//! let mut rng = Pcg64Mcg::seed_from_u64(0);
//! let x: u32 = rng.next_u32();
//! ```
//!
//! The functionality of this crate is implemented using traits from the `rand_core` crate, but you may use the `rand`
//! crate for further functionality to initialize the generator from various sources and to generate random values:
//!
//! ```ignore
//! use rand::{Rng, SeedableRng};
//! use rand_pcg::Pcg64Mcg;
//!
//! let mut rng = Pcg64Mcg::from_entropy();
//! let x: f64 = rng.gen();
//! ```

#![doc(
    html_logo_url = "https://www.rust-lang.org/logos/rust-logo-128x128-blk.png",
    html_favicon_url = "https://www.rust-lang.org/favicon.ico",
    html_root_url = "https://rust-random.github.io/rand/"
)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![no_std]

mod pcg128;
mod pcg128cm;
mod pcg64;

pub use self::pcg128::{Lcg128Xsl64, Mcg128Xsl64, Pcg64, Pcg64Mcg};
pub use self::pcg128cm::{Lcg128CmDxsm64, Pcg64Dxsm};
pub use self::pcg64::{Lcg64Xsh32, Pcg32};
