# Rand

[![Test Status](https://github.com/rust-random/rand/workflows/Tests/badge.svg?event=push)](https://github.com/rust-random/rand/actions)
[![Crate](https://img.shields.io/crates/v/rand.svg)](https://crates.io/crates/rand)
[![Book](https://img.shields.io/badge/book-master-yellow.svg)](https://rust-random.github.io/book/)
[![API](https://img.shields.io/badge/api-master-yellow.svg)](https://rust-random.github.io/rand/rand)
[![API](https://docs.rs/rand/badge.svg)](https://docs.rs/rand)

Rand is a Rust library supporting random generators:

-   A standard RNG trait: [`rand_core::RngCore`](https://docs.rs/rand_core/latest/rand_core/trait.RngCore.html)
-   Fast implementations of the best-in-class [cryptographic](https://rust-random.github.io/book/guide-rngs.html#cryptographically-secure-pseudo-random-number-generators-csprngs) and
    [non-cryptographic](https://rust-random.github.io/book/guide-rngs.html#basic-pseudo-random-number-generators-prngs) generators: [`rand::rngs`](https://docs.rs/rand/latest/rand/rngs/index.html), and more RNGs: [`rand_chacha`](https://docs.rs/rand_chacha), [`rand_xoshiro`](https://docs.rs/rand_xoshiro/), [`rand_pcg`](https://docs.rs/rand_pcg/), [rngs repo](https://github.com/rust-random/rngs/)
-   [`rand::rng`](https://docs.rs/rand/latest/rand/fn.rng.html) is an asymtotically-fast, reasonably secure generator available on all `std` targets
-   Secure seeding via the [`getrandom` crate](https://crates.io/crates/getrandom)

Supporting random value generation and random processes:

-   [`Standard`](https://docs.rs/rand/latest/rand/distributions/struct.Standard.html) random value generation
-   Ranged [`Uniform`](https://docs.rs/rand/latest/rand/distributions/struct.Uniform.html) number generation for many types
-   A flexible [`distributions`](https://docs.rs/rand/*/rand/distr/index.html) module
-   Samplers for a large number of random number distributions via our own
    [`rand_distr`](https://docs.rs/rand_distr) and via
    the [`statrs`](https://docs.rs/statrs/0.13.0/statrs/)
-   Random processes (mostly choose and shuffle) via [`rand::seq`](https://docs.rs/rand/latest/rand/seq/index.html) traits

All with:

-   [Portably reproducible output](https://rust-random.github.io/book/portability.html)
-   `#[no_std]` compatibility (partial)
-   *Many* performance optimisations

It's also worth pointing out what Rand *is not*:

-   Small. Most low-level crates are small, but the higher-level `rand` and
    `rand_distr` each contain a lot of functionality.
-   Simple (implementation). We have a strong focus on correctness, speed and flexibility, but
    not simplicity. If you prefer a small-and-simple library, there are
    alternatives including [fastrand](https://crates.io/crates/fastrand)
    and [oorandom](https://crates.io/crates/oorandom).
-   Slow. We take performance seriously, with considerations also for set-up
    time of new distributions, commonly-used parameters, and parameters of the
    current sampler.

Documentation:

-   [The Rust Rand Book](https://rust-random.github.io/book)
-   [API reference (master branch)](https://rust-random.github.io/rand)
-   [API reference (docs.rs)](https://docs.rs/rand)


## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
rand = "0.8.5"
```

To get started using Rand, see [The Book](https://rust-random.github.io/book).

## Versions

Rand is *mature* (suitable for general usage, with infrequent breaking releases
which minimise breakage) but not yet at 1.0. Current versions are:

-   Version 0.8 was released in December 2020 with many small changes.
-   Version 0.9 is in development with many small changes.

See the [CHANGELOG](CHANGELOG.md) or [Upgrade Guide](https://rust-random.github.io/book/update.html) for more details.

## Crate Features

Rand is built with these features enabled by default:

-   `std` enables functionality dependent on the `std` lib
-   `alloc` (implied by `std`) enables functionality requiring an allocator
-   `getrandom` (implied by `std`) is an optional dependency providing the code
    behind `rngs::OsRng`
-   `std_rng` enables inclusion of `StdRng`, `ThreadRng`

Optionally, the following dependencies can be enabled:

-   `log` enables logging via [log](https://crates.io/crates/log)

Additionally, these features configure Rand:

-   `small_rng` enables inclusion of the `SmallRng` PRNG
-   `nightly` includes some additions requiring nightly Rust
-   `simd_support` (experimental) enables sampling of SIMD values
    (uniformly random SIMD integers and floats), requiring nightly Rust

Note that nightly features are not stable and therefore not all library and
compiler versions will be compatible. This is especially true of Rand's
experimental `simd_support` feature.

Rand supports limited functionality in `no_std` mode (enabled via
`default-features = false`). In this case, `OsRng` and `from_os_rng` are
unavailable (unless `getrandom` is enabled), large parts of `seq` are
unavailable (unless `alloc` is enabled), and `ThreadRng` is unavailable.

## Portability and platform support

Many (but not all) algorithms are intended to have reproducible output. Read more in the book: [Portability](https://rust-random.github.io/book/portability.html).

The Rand library supports a variety of CPU architectures. Platform integration is outsourced to [getrandom](https://docs.rs/getrandom/latest/getrandom/).

### WASM support

Seeding entropy from OS on WASM target `wasm32-unknown-unknown` is not
*automatically* supported by `rand` or `getrandom`. If you are fine with
seeding the generator manually, you can disable the `getrandom` feature
and use the methods on the `SeedableRng` trait. To enable seeding from OS,
either use a different target such as `wasm32-wasi` or add a direct
dependency on `getrandom` with the `js` feature (if the target supports
JavaScript). See
[getrandom#WebAssembly support](https://docs.rs/getrandom/latest/getrandom/#webassembly-support).

# License

Rand is distributed under the terms of both the MIT license and the
Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT), and
[COPYRIGHT](COPYRIGHT) for details.
