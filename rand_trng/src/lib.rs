// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Thread-local random number generator

use core::cell::UnsafeCell;
use std::fmt;
use std::rc::Rc;
use std::thread_local;

pub use rand_chacha::rand_core::{self, CryptoRng, RngCore};

use rand_chacha::{rand_core::SeedableRng, ChaCha12Rng};

// Number of generated bytes after which to reseed `ThreadRng`.
// According to benchmarks, reseeding has a noticeable impact with thresholds
// of 32 kB and less. We choose 64 kB to avoid significant overhead.
const RESEED_THRESHOLD: isize = 1024 * 64;

struct InnerState {
    rng: ChaCha12Rng,
    bytes_until_reseed: isize,
}

impl InnerState {
    #[inline(always)]
    fn reseed(&mut self) -> Result<(), rand_core::getrandom::Error> {
        self.bytes_until_reseed = RESEED_THRESHOLD;
        self.rng = ChaCha12Rng::try_from_os_rng()?;
        Ok(())
    }

    #[inline(always)]
    fn reseed_check(&mut self, n: isize) {
        if self.bytes_until_reseed < 0 {
            // If system RNG has failed for some reason, ignore the error
            // and continue to work with the old RNG state.
            let _ = self.reseed();
        }
        self.bytes_until_reseed -= n;
    }
}

thread_local!(
    // We require Rc<..> to avoid premature freeing when ThreadRng is used
    // within thread-local destructors. See https://github.com/rust-random/rand/issues/968.
    //
    // Rationale for using `UnsafeCell`:
    //
    // Previously we used a `RefCell`, with an overhead of ~15%. There will only
    // ever be one mutable reference to the interior of the `UnsafeCell`, because
    // we only have such a reference inside `next_u32`, `next_u64`, etc. Within a
    // single thread (which is the definition of `ThreadRng`), there will only ever
    // be one of these methods active at a time.
    //
    // A possible scenario where there could be multiple mutable references is if
    // `ThreadRng` is used inside `next_u32` and co. But the implementation is
    // completely under our control. We just have to ensure none of them use
    // `ThreadRng` internally, which is nonsensical anyway. We should also never run
    // `ThreadRng` in destructors of its implementation, which is also nonsensical.
    static THREAD_RNG_KEY: Rc<UnsafeCell<InnerState>> = {
        let rng = match ChaCha12Rng::try_from_os_rng() {
            Ok(rng) => rng,
            Err(err) => panic!("could not initialize ThreadRng: {err}"),
        };
        Rc::new(UnsafeCell::new(InnerState { rng, bytes_until_reseed: RESEED_THRESHOLD }))
    }
);

/// A reference to the thread-local generator.
///
/// This type is a reference to a lazily-initialized thread-local generator.
/// An instance can be obtained via [`ThreadRng::new()`] or [`ThreadRng::default()`].
/// The handle cannot be passed between threads (is not [`Send`] or [`Sync`]).
///
/// # Example
///
/// ```
/// use rand_trng::{ThreadRng, RngCore};
///
/// let mut rng = ThreadRng::new();
///
/// let random_u32 = rng.next_u32();
/// let random_u64 = rng.next_u64();
///
/// let mut buf = [0u8; 32];
/// rng.fill_bytes(&mut buf);
/// ```
///
/// # Security
///
/// Security must be considered relative to a threat model and validation
/// requirements. The Rand project can provide no guarantee of fitness for
/// purpose. The design criteria for `ThreadRng` are as follows:
///
/// - Automatic seeding via [`OsRng`] and periodically thereafter after every 64 KiB of
///   generated data. Limitation: there is no automatic
///   reseeding on process fork (see [below](#fork)).
/// - A rigorusly analyzed, unpredictable (cryptographic) pseudo-random generator
///   (see [the book on security](https://rust-random.github.io/book/guide-rngs.html#security)).
///   The currently selected algorithm is ChaCha (12-rounds).
/// - Not to leak internal state through [`Debug`] or serialization implementations.
/// - No further protections exist to in-memory state. In particular, the
///   implementation is not required to zero memory on exit (of the process or
///   thread). (This may change in the future.)
/// - Be fast enough for general-purpose usage. Note in particular that
///   `ThreadRng` is designed to be a "fast, reasonably secure generator"
///   (where "reasonably secure" implies the above criteria).
///
/// We leave it to the user to determine whether this generator meets their
/// security requirements. For an alternative, see [`OsRng`].
///
/// # Fork
///
/// `ThreadRng` is not automatically reseeded on fork. It is recommended to
/// explicitly call [`ThreadRng::reseed`] immediately after a fork, for example:
/// ```ignore
/// fn do_fork() {
///     let pid = unsafe { libc::fork() };
///     if pid == 0 {
///         // Reseed ThreadRng in child processes:
///         rand::rng().reseed();
///     }
/// }
/// ```
///
/// Methods on `ThreadRng` are not reentrant-safe and thus should not be called
/// from an interrupt (e.g. a fork handler) unless it can be guaranteed that no
/// other method on the same `ThreadRng` is currently executing.
///
/// [`OsRng`]: rand_core::OsRng
#[derive(Clone)]
pub struct ThreadRng {
    rng: Rc<UnsafeCell<InnerState>>,
}

impl ThreadRng {
    /// Create a reference to the thread-local generator.
    pub fn new() -> Self {
        Default::default()
    }

    /// Immediately reseed the generator
    ///
    /// This discards any remaining random data in the cache.
    pub fn reseed(&mut self) -> Result<(), rand_core::getrandom::Error> {
        // SAFETY: The state is thread-local and the reference does not leak from this method
        let s = unsafe { &mut *self.rng.get() };
        s.reseed()
    }
}

/// Debug implementation does not leak internal state
impl fmt::Debug for ThreadRng {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("ThreadRng { .. }")
    }
}

impl Default for ThreadRng {
    fn default() -> Self {
        let rng = THREAD_RNG_KEY.with(|t| t.clone());
        Self { rng }
    }
}

impl RngCore for ThreadRng {
    #[inline(always)]
    fn next_u32(&mut self) -> u32 {
        // SAFETY: The state is thread-local and the reference does not leak from this method
        let s = unsafe { &mut *self.rng.get() };
        s.reseed_check(core::mem::size_of::<u32>() as isize);
        s.rng.next_u32()
    }

    #[inline(always)]
    fn next_u64(&mut self) -> u64 {
        // SAFETY: The state is thread-local and the reference does not leak from this method
        let s = unsafe { &mut *self.rng.get() };
        s.reseed_check(core::mem::size_of::<u64>() as isize);
        s.rng.next_u64()
    }

    #[inline(always)]
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        // SAFETY: The state is thread-local and the reference does not leak from this method
        let s = unsafe { &mut *self.rng.get() };
        // Valid allocations can not be bigger than `isize::MAX` bytes,
        // so we can cast length to `isize` without issues.
        s.reseed_check(dest.len() as isize);
        s.rng.fill_bytes(dest)
    }
}

impl CryptoRng for ThreadRng {}
