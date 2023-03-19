// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Math helper functions

#[cfg(feature = "simd_support")] use core::simd::*;

pub(crate) trait WideningMultiply<RHS = Self> {
    type Output;

    fn wmul(self, x: RHS) -> Self::Output;
}

macro_rules! wmul_impl {
    ($ty:ty, $wide:ty, $shift:expr) => {
        impl WideningMultiply for $ty {
            type Output = ($ty, $ty);

            #[inline(always)]
            fn wmul(self, x: $ty) -> Self::Output {
                let tmp = (self as $wide) * (x as $wide);
                ((tmp >> $shift) as $ty, tmp as $ty)
            }
        }
    };

    // simd bulk implementation
    ($(($ty:ident, $wide:ty),)+, $shift:expr) => {
        $(
            impl WideningMultiply for $ty {
                type Output = ($ty, $ty);

                #[inline(always)]
                fn wmul(self, x: $ty) -> Self::Output {
                    // For supported vectors, this should compile to a couple
                    // supported multiply & swizzle instructions (no actual
                    // casting).
                    // TODO: optimize
                    let y: $wide = self.cast();
                    let x: $wide = x.cast();
                    let tmp = y * x;
                    let hi: $ty = (tmp >> Simd::splat($shift)).cast();
                    let lo: $ty = tmp.cast();
                    (hi, lo)
                }
            }
        )+
    };
}
wmul_impl! { u8, u16, 8 }
wmul_impl! { u16, u32, 16 }
wmul_impl! { u32, u64, 32 }
wmul_impl! { u64, u128, 64 }

// This code is a translation of the __mulddi3 function in LLVM's
// compiler-rt. It is an optimised variant of the common method
// `(a + b) * (c + d) = ac + ad + bc + bd`.
//
// For some reason LLVM can optimise the C version very well, but
// keeps shuffling registers in this Rust translation.
macro_rules! wmul_impl_large {
    ($ty:ty, $half:expr) => {
        impl WideningMultiply for $ty {
            type Output = ($ty, $ty);

            #[inline(always)]
            fn wmul(self, b: $ty) -> Self::Output {
                const LOWER_MASK: $ty = !0 >> $half;
                let mut low = (self & LOWER_MASK).wrapping_mul(b & LOWER_MASK);
                let mut t = low >> $half;
                low &= LOWER_MASK;
                t += (self >> $half).wrapping_mul(b & LOWER_MASK);
                low += (t & LOWER_MASK) << $half;
                let mut high = t >> $half;
                t = low >> $half;
                low &= LOWER_MASK;
                t += (b >> $half).wrapping_mul(self & LOWER_MASK);
                low += (t & LOWER_MASK) << $half;
                high += t >> $half;
                high += (self >> $half).wrapping_mul(b >> $half);

                (high, low)
            }
        }
    };

    // simd bulk implementation
    (($($ty:ty,)+) $scalar:ty, $half:expr) => {
        $(
            impl WideningMultiply for $ty {
                type Output = ($ty, $ty);

                #[inline(always)]
                fn wmul(self, b: $ty) -> Self::Output {
                    // needs wrapping multiplication
                    let lower_mask = <$ty>::splat(!0 >> $half);
                    let half = <$ty>::splat($half);
                    let mut low = (self & lower_mask) * (b & lower_mask);
                    let mut t = low >> half;
                    low &= lower_mask;
                    t += (self >> half) * (b & lower_mask);
                    low += (t & lower_mask) << half;
                    let mut high = t >> half;
                    t = low >> half;
                    low &= lower_mask;
                    t += (b >> half) * (self & lower_mask);
                    low += (t & lower_mask) << half;
                    high += t >> half;
                    high += (self >> half) * (b >> half);

                    (high, low)
                }
            }
        )+
    };
}
wmul_impl_large! { u128, 64 }

macro_rules! wmul_impl_usize {
    ($ty:ty) => {
        impl WideningMultiply for usize {
            type Output = (usize, usize);

            #[inline(always)]
            fn wmul(self, x: usize) -> Self::Output {
                let (high, low) = (self as $ty).wmul(x as $ty);
                (high as usize, low as usize)
            }
        }
    };
}
#[cfg(target_pointer_width = "16")]
wmul_impl_usize! { u16 }
#[cfg(target_pointer_width = "32")]
wmul_impl_usize! { u32 }
#[cfg(target_pointer_width = "64")]
wmul_impl_usize! { u64 }

#[cfg(feature = "simd_support")]
mod simd_wmul {
    use super::*;
    #[cfg(target_arch = "x86")] use core::arch::x86::*;
    #[cfg(target_arch = "x86_64")] use core::arch::x86_64::*;

    wmul_impl! {
        (u8x4, u16x4),
        (u8x8, u16x8),
        (u8x16, u16x16),
        (u8x32, u16x32),
        (u8x64, Simd<u16, 64>),,
        8
    }

    wmul_impl! { (u16x2, u32x2),, 16 }
    wmul_impl! { (u16x4, u32x4),, 16 }
    #[cfg(not(target_feature = "sse2"))]
    wmul_impl! { (u16x8, u32x8),, 16 }
    #[cfg(not(target_feature = "avx2"))]
    wmul_impl! { (u16x16, u32x16),, 16 }
    #[cfg(not(target_feature = "avx512bw"))]
    wmul_impl! { (u16x32, Simd<u32, 32>),, 16 }

    // 16-bit lane widths allow use of the x86 `mulhi` instructions, which
    // means `wmul` can be implemented with only two instructions.
    #[allow(unused_macros)]
    macro_rules! wmul_impl_16 {
        ($ty:ident, $mulhi:ident, $mullo:ident) => {
            impl WideningMultiply for $ty {
                type Output = ($ty, $ty);

                #[inline(always)]
                fn wmul(self, x: $ty) -> Self::Output {
                    let hi = unsafe { $mulhi(self.into(), x.into()) }.into();
                    let lo = unsafe { $mullo(self.into(), x.into()) }.into();
                    (hi, lo)
                }
            }
        };
    }

    #[cfg(target_feature = "sse2")]
    wmul_impl_16! { u16x8, _mm_mulhi_epu16, _mm_mullo_epi16 }
    #[cfg(target_feature = "avx2")]
    wmul_impl_16! { u16x16, _mm256_mulhi_epu16, _mm256_mullo_epi16 }
    #[cfg(target_feature = "avx512bw")]
    wmul_impl_16! { u16x32, _mm512_mulhi_epu16, _mm512_mullo_epi16 }

    wmul_impl! {
        (u32x2, u64x2),
        (u32x4, u64x4),
        (u32x8, u64x8),
        (u32x16, Simd<u64, 16>),,
        32
    }

    wmul_impl_large! { (u64x2, u64x4, u64x8,) u64, 32 }
}

/// Helper trait when dealing with scalar and SIMD floating point types.
pub(crate) trait FloatSIMDUtils {
    // `PartialOrd` for vectors compares lexicographically. We want to compare all
    // the individual SIMD lanes instead, and get the combined result over all
    // lanes. This is possible using something like `a.lt(b).all()`, but we
    // implement it as a trait so we can write the same code for `f32` and `f64`.
    // Only the comparison functions we need are implemented.
    fn all_lt(self, other: Self) -> bool;
    fn all_le(self, other: Self) -> bool;
    fn all_finite(self) -> bool;

    type Mask;
    fn finite_mask(self) -> Self::Mask;
    fn gt_mask(self, other: Self) -> Self::Mask;
    fn ge_mask(self, other: Self) -> Self::Mask;
    fn lt_mask(self, other: Self) -> Self::Mask;

    // Decrease all lanes where the mask is `true` to the next lower value
    // representable by the floating-point type. At least one of the lanes
    // must be set.
    fn decrease_masked(self, mask: Self::Mask) -> Self;

    // Increase all lanes where the mask is `true` to the next higher value
    // representable by the floating-point type. At least one of the lanes
    // must be set.
    fn increase_masked(self, mask: Self::Mask) -> Self;

    // Similar to the proposed `next_down()` method for rust's f32 and f64,
    // but this implementation does not handle `inf` or `nan`.
    fn utils_next_down(self) -> Self;

    // Convert from int value. Conversion is done while retaining the numerical
    // value, not by retaining the binary representation.
    type UInt;
    fn cast_from_int(i: Self::UInt) -> Self;

    type Scalar;
    fn replace(self, index: usize, new_value: Self::Scalar) -> Self;
    fn extract(self, index: usize) -> Self::Scalar;
}

/// Implement functions available in std builds but missing from core primitives
#[cfg(not(std))]
// False positive: We are following `std` here.
#[allow(clippy::wrong_self_convention)]
pub(crate) trait Float: Sized {
    fn is_nan(self) -> bool;
    fn is_infinite(self) -> bool;
    fn is_finite(self) -> bool;
}

/// Implement functions on f32/f64 to give them APIs similar to SIMD types
pub(crate) trait FloatAsSIMD: Sized {
    const LANES: usize = 1;
    #[inline(always)]
    fn splat(scalar: Self) -> Self {
        scalar
    }
}

pub(crate) trait IntAsSIMD: Sized {
    #[inline(always)]
    fn splat(scalar: Self) -> Self {
        scalar
    }
}

impl IntAsSIMD for u32 {}
impl IntAsSIMD for u64 {}

pub(crate) trait BoolAsSIMD: Sized {
    fn any(self) -> bool;
    fn all(self) -> bool;
    fn none(self) -> bool;
}

impl BoolAsSIMD for bool {
    #[inline(always)]
    fn any(self) -> bool {
        self
    }

    #[inline(always)]
    fn all(self) -> bool {
        self
    }

    #[inline(always)]
    fn none(self) -> bool {
        !self
    }
}

macro_rules! scalar_float_impl {
    ($ty:ident, $uty:ident) => {
        #[cfg(not(std))]
        impl Float for $ty {
            #[inline]
            fn is_nan(self) -> bool {
                self != self
            }

            #[inline]
            fn is_infinite(self) -> bool {
                self == ::core::$ty::INFINITY || self == ::core::$ty::NEG_INFINITY
            }

            #[inline]
            fn is_finite(self) -> bool {
                !(self.is_nan() || self.is_infinite())
            }
        }

        impl FloatSIMDUtils for $ty {
            type Mask = bool;
            type Scalar = $ty;
            type UInt = $uty;

            #[inline(always)]
            fn all_lt(self, other: Self) -> bool {
                self < other
            }

            #[inline(always)]
            fn all_le(self, other: Self) -> bool {
                self <= other
            }

            #[inline(always)]
            fn all_finite(self) -> bool {
                self.is_finite()
            }

            #[inline(always)]
            fn finite_mask(self) -> Self::Mask {
                self.is_finite()
            }

            #[inline(always)]
            fn gt_mask(self, other: Self) -> Self::Mask {
                self > other
            }

            #[inline(always)]
            fn ge_mask(self, other: Self) -> Self::Mask {
                self >= other
            }

            #[inline(always)]
            fn lt_mask(self, other: Self) -> Self::Mask {
                self < other
            }

            #[inline(always)]
            fn decrease_masked(self, mask: Self::Mask) -> Self {
                debug_assert!(mask, "At least one lane must be set");
                <$ty>::from_bits(self.to_bits() - 1)
            }

            #[inline(always)]
            fn increase_masked(self, mask: Self::Mask) -> Self {
                debug_assert!(mask, "At least one lane must be set");
                <$ty>::from_bits(self.to_bits() + 1)
            }

            #[inline(always)]
            fn utils_next_down(self) -> Self {
                // This is not a drop-in replacement for the next_down() method
                // proposed for rust (https://github.com/rust-lang/rust/issues/91399).
                // This functions assumes that the input is not nan or inf.
                if self > 0.0 {
                    <$ty>::from_bits(self.to_bits() - 1)
                } else if self < 0.0 {
                    <$ty>::from_bits(self.to_bits() + 1)
                } else {
                    <$ty>::from_bits((-0.0 as $ty).to_bits() + 1)
                }
            }

            #[inline]
            fn cast_from_int(i: Self::UInt) -> Self {
                i as $ty
            }

            #[inline]
            fn replace(self, index: usize, new_value: Self::Scalar) -> Self {
                debug_assert_eq!(index, 0);
                new_value
            }

            #[inline]
            fn extract(self, index: usize) -> Self::Scalar {
                debug_assert_eq!(index, 0);
                self
            }
        }

        impl FloatAsSIMD for $ty {}
    };
}

scalar_float_impl!(f32, u32);
scalar_float_impl!(f64, u64);


#[cfg(feature = "simd_support")]
macro_rules! simd_impl {
    ($fty:ident, $uty:ident) => {
        impl<const LANES: usize> FloatSIMDUtils for Simd<$fty, LANES>
        where LaneCount<LANES>: SupportedLaneCount
        {
            type Mask = Mask<<$fty as SimdElement>::Mask, LANES>;
            type Scalar = $fty;
            type UInt = Simd<$uty, LANES>;

            #[inline(always)]
            fn all_lt(self, other: Self) -> bool {
                self.simd_lt(other).all()
            }

            #[inline(always)]
            fn all_le(self, other: Self) -> bool {
                self.simd_le(other).all()
            }

            #[inline(always)]
            fn all_finite(self) -> bool {
                self.is_finite().all()
            }

            #[inline(always)]
            fn finite_mask(self) -> Self::Mask {
                self.is_finite()
            }

            #[inline(always)]
            fn gt_mask(self, other: Self) -> Self::Mask {
                self.simd_gt(other)
            }

            #[inline(always)]
            fn ge_mask(self, other: Self) -> Self::Mask {
                self.simd_ge(other)
            }

            #[inline(always)]
            fn lt_mask(self, other: Self) -> Self::Mask {
                self.simd_lt(other)
            }

            #[inline(always)]
            fn decrease_masked(self, mask: Self::Mask) -> Self {
                // Casting a mask into ints will produce all bits set for
                // true, and 0 for false. Adding that to the binary
                // representation of a float means subtracting one from
                // the binary representation, resulting in the next lower
                // value representable by $fty. This works even when the
                // current value is infinity.
                debug_assert!(mask.any(), "At least one lane must be set");
                Self::from_bits(self.to_bits() + mask.to_int().cast())
            }

            #[inline(always)]
            fn increase_masked(self, mask: Self::Mask) -> Self {
                debug_assert!(mask.any(), "At least one lane must be set");
                let zero = Simd::<$uty, LANES>::splat(0);
                let one = Simd::<$uty, LANES>::splat(1);
                Self::from_bits(self.to_bits() + mask.select(one, zero))
            }

            #[inline(always)]
            fn utils_next_down(self) -> Self {
                // This is not a drop-in replacement for the next_down() method
                // proposed for rust (https://github.com/rust-lang/rust/issues/91399).
                // This function assumes that no values are nan or inf.
                let zero = Self::splat(0.0 as $fty);
                let pos_mask = self.simd_gt(zero);
                let neg_mask = self.simd_lt(zero);
                let zero_mask = self.simd_eq(zero); // Could be +0.0 or -0.0
                let mut bits = self.to_bits();
                bits += pos_mask.to_int().cast();
                bits += (-neg_mask.to_int()).cast();
                // Shenanigans so both +0.0 and -0.0 end up as next_down(0.0).
                // The bit patterns for 0.0 and -0.0 are 000...000 and 100...000, resp.
                // We want both of these to result in 100...001.
                let zero_next_down = (1 << ($uty::BITS - 1)) | 1; // This is 100...001
                bits |= (zero_mask.to_int().cast()) & Simd::<$uty, LANES>::splat(zero_next_down);
                Self::from_bits(bits)
            }

            #[inline]
            fn cast_from_int(i: Self::UInt) -> Self {
                i.cast()
            }

            #[inline]
            fn replace(mut self, index: usize, new_value: Self::Scalar) -> Self {
                self.as_mut_array()[index] = new_value;
                self
            }

            #[inline]
            fn extract(self, index: usize) -> Self::Scalar {
                self.as_array()[index]
            }
        }
    };
}

#[cfg(feature = "simd_support")]
simd_impl!(f32, u32);
#[cfg(feature = "simd_support")]
simd_impl!(f64, u64);

trait Summable<T> {
    fn compensated_sum(&self) -> T;
}

pub(crate) trait ScaleComputable<T> {
    // This utility function computes the `scale` for the uniform float
    // distribution.  It ensures that
    //    scale * max_rand + low < high
    // and
    //    next_up(scale) * max_rand + low >= high
    // where `max_rand` is the maximum value that can be generated by
    // the uniform distribution on [0, 1) and `next_up(scale)` is the
    // smallest float that is larger than `scale`.
    fn compute_scale(low: T, high: T) -> T;
}

macro_rules! uniform_compute_scale_impl {
    ($ty:ty, $f_scalar:ident) => {
        impl Summable<$ty> for &[$ty] {
            fn compensated_sum(&self) -> $ty {
                // Kahan compensated sum
                let mut sum = <$ty>::splat(0.0);
                let mut c = <$ty>::splat(0.0);
                for val in *self {
                    let y = val - c;
                    let t = sum + y;
                    c = (t - sum) - y;
                    sum = t;
                }
                sum
            }
        }

        impl ScaleComputable<$ty> for $ty {
            fn compute_scale(low: $ty, high: $ty) -> $ty {
                let eps = <$ty>::splat($f_scalar::EPSILON);

                // `max_rand` is 1.0 - eps.  This is actually the second largest
                // float less than 1.0, because the spacing of the floats in the
                // interval [0.5, 1.0) is `eps/2`.
                let max_rand = <$ty>::splat(1.0 as $f_scalar - $f_scalar::EPSILON);

                // `delta_high` is half the distance from `high` to the largest
                // float that is less than `high`.  If `high` is subnormal or 0,
                // then `delta_high` will be 0. Why this is needed is explained
                // below.
                let delta_high = <$ty>::splat(0.5) * (high - high.utils_next_down());

                // We want `scale * max_rand + low < high`. Let `high_1` be the
                // (hypothetical) float that is the midpoint between `high` and
                // the largest float less than `high`.  The midpoint is used
                // because any float calculation  that would result in a value in
                // `(high_1, high)` would be rounded to `high`.   The ideal
                // condition for upper bound of `scale` is then
                //     scale * max_rand + low = high_1`
                // or
                //     scale = (high_1 - low)/max_rand
                //
                // Write `high_1 = high - delta_high`,  `max_rand = 1 - eps`,
                // and approximate `1/(1 - eps)` as `(1 + eps)`.  Then we have
                //
                //     scale = (high - delta_high - low)*(1 + eps)
                //           = high - low + eps*high - eps*low - delta_high
                //
                // (The extremely small term `-delta_high*eps` has been ignored.)
                // The following uses Kahan's compensated summation to compute `scale`
                // from those terms.
                let terms: &[$ty] = &[high, -low, eps * high, -eps * low, -delta_high];
                let mut scale = terms.compensated_sum();

                // Empirical tests show that `scale` is generally within 1 or 2 ULPs
                // of the "ideal" scale.  Next we adjust `scale`, if necessary, to
                // the ideal value.

                // Check that `scale * max_rand + low` is less than `high`. If it is
                // not, repeatedly adjust `scale` down by one ULP until the condition
                // is satisfied. Generally this requires 0 or 1 adjustments to `scale`.
                // (The original `too_big_mask` is saved so we can use it again below.)
                let too_big_mask = (scale * max_rand + low).ge_mask(high);
                loop {
                    let mask = (scale * max_rand + low).ge_mask(high);
                    if !mask.any() {
                        break;
                    }
                    scale = scale.decrease_masked(mask);
                }
                // We have ensured that `scale * max_rand + low < high`.  Now see if
                // we can increase `scale` and still maintain that inequality.  We
                // only need to do this if `scale` was not initially too big.
                let not_too_big_mask = !too_big_mask;
                let mut mask = not_too_big_mask;
                if mask.any() {
                    loop {
                        let next_scale = scale.increase_masked(mask);
                        mask = (next_scale * max_rand + low).lt_mask(high) & not_too_big_mask;
                        if !mask.any() {
                            break;
                        }
                        scale = scale.increase_masked(mask);
                    }
                }
                scale
            }
        }
    };
}

uniform_compute_scale_impl! { f32, f32 }
uniform_compute_scale_impl! { f64, f64 }

#[cfg(feature = "simd_support")]
uniform_compute_scale_impl! { f32x2, f32 }
#[cfg(feature = "simd_support")]
uniform_compute_scale_impl! { f32x4, f32 }
#[cfg(feature = "simd_support")]
uniform_compute_scale_impl! { f32x8, f32 }
#[cfg(feature = "simd_support")]
uniform_compute_scale_impl! { f32x16, f32 }

#[cfg(feature = "simd_support")]
uniform_compute_scale_impl! { f64x2, f64 }
#[cfg(feature = "simd_support")]
uniform_compute_scale_impl! { f64x4, f64 }
#[cfg(feature = "simd_support")]
uniform_compute_scale_impl! { f64x8, f64 }


#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! scalar_increase_masked_tests {
        ($($fname:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $fname() {
                    let (input, expected) = $value;
                    assert_eq!(input.increase_masked(true), expected);
                }
            )*
        }
    }

    scalar_increase_masked_tests! {
        scalar_increase_masked_case0: (1.0f64, 1.0f64 + f64::EPSILON),
        scalar_increase_masked_case1: (1.0f32, 1.0f32 + f32::EPSILON),
        scalar_increase_masked_case2: (0.625f64, 0.625f64 + 0.5f64 * f64::EPSILON),
        scalar_increase_masked_case3: (0.625f32, 0.625f32 + 0.5f32 * f32::EPSILON),
        scalar_increase_masked_case4: (f64::from_bits(1), f64::from_bits(2)),
        scalar_increase_masked_case5: (f32::from_bits(1), f32::from_bits(2)),
        scalar_increase_masked_case6: (0.0f64, f64::from_bits(1)),
        scalar_increase_masked_case7: (0.0f32, f32::from_bits(1)),
    }

    macro_rules! simd_increase_masked_tests {
        ($($fname:ident: ($ty:ty, $f_scalar:ident),)*) => {
            $(
                #[test]
                #[cfg(feature = "simd_support")]
                fn $fname() {
                    let values = [
                        10.5 as $f_scalar, 1.0 as $f_scalar, 1.0e-3 as $f_scalar, $f_scalar::from_bits(1), 0.0 as $f_scalar
                    ];
                    for input0 in values {
                        let x = <$ty>::splat(input0);
                        // Create a mask with just the first lane set.
                        let mut mask = Mask::from_array([false; <$ty>::LANES]);
                        mask.set(0, true);

                        let y = x.increase_masked(mask);

                        // Independently create the expected result, based on applying
                        // increase_masked() to the scalar value in channel 0.
                        let mut xa = x.to_array();
                        xa[0] = xa[0].increase_masked(true);
                        let expected = Simd::from_array(xa);

                        assert_eq!(y, expected);
                    }
                }
            )*
        }
    }

    simd_increase_masked_tests! {
        simd_increase_masked_case0: (f32x2, f32),
        simd_increase_masked_case1: (f32x4, f32),
        simd_increase_masked_case2: (f32x8, f32),
        simd_increase_masked_case3: (f32x16, f32),
        simd_increase_masked_case4: (f64x2, f64),
        simd_increase_masked_case5: (f64x4, f64),
        simd_increase_masked_case6: (f64x8, f64),
    }

    macro_rules! scalar_utils_next_down_tests {
        ($($fname:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $fname() {
                    let (input, expected) = $value;
                    assert_eq!(input.utils_next_down(), expected);
                }
            )*
        }
    }

    scalar_utils_next_down_tests! {
        utils_next_down_case0: (3000.0f32, 2999.9998f32),
        utils_next_down_case1: (-3000.0f32, -3000.0002f32),
        utils_next_down_case2: (3000.0f64, 2999.9999999999995f64),
        utils_next_down_case3: (-3000.0f64, -3000.0000000000005f64),
        utils_next_down_case4: (1.0f64, 1.0f64 - 0.5f64 * f64::EPSILON),
        utils_next_down_case5: (1.0f32, 1.0f32 - 0.5f32 * f32::EPSILON),
        utils_next_down_case6: (-1.0f64, -1.0f64 - f64::EPSILON),
        utils_next_down_case7: (-1.0f32, -1.0f32 - f32::EPSILON),
        utils_next_down_case8: (0.625f64, 0.625f64 - 0.5f64 * f64::EPSILON),
        utils_next_down_case9: (0.625f32, 0.625f32 - 0.5f32 * f32::EPSILON),
        utils_next_down_case10: (-0.625f64, -0.625f64 - 0.5f64 * f64::EPSILON),
        utils_next_down_case11: (-0.625f32, -0.625f32 - 0.5f32 * f32::EPSILON),
        utils_next_down_case12: (f64::from_bits(2), f64::from_bits(1)),
        utils_next_down_case13: (f32::from_bits(2), f32::from_bits(1)),
        utils_next_down_case14: (f64::from_bits(1), 0.0f64),
        utils_next_down_case15: (f32::from_bits(1), 0.0f32),
        utils_next_down_case16: (0.0f64, -f64::from_bits(1)),
        utils_next_down_case17: (0.0f32, -f32::from_bits(1)),
        utils_next_down_case18: (-0.0f64, -f64::from_bits(1)),
        utils_next_down_case19: (-0.0f32, -f32::from_bits(1)),
    }

    macro_rules! simd_utils_next_down_tests {
        ($($fname:ident: ($ty:ty, $f_scalar:ident),)*) => {
            $(
                #[test]
                #[cfg(feature = "simd_support")]
                fn $fname() {
                    let values = [
                        10.5 as $f_scalar, 1.0 as $f_scalar, 1.0e-3 as $f_scalar, $f_scalar::from_bits(1), 0.0 as $f_scalar,
                        -10.5 as $f_scalar, -1.0 as $f_scalar, -1.0e-3 as $f_scalar, -$f_scalar::from_bits(1), -0.0 as $f_scalar,
                    ];
                    // Test that the vector version gives the same results as
                    // the scalar version.  Create test vectors that use two
                    // values at a time from `values`.
                    for k in 0..(values.len() - 1) {
                        let c1 = <$ty>::splat(values[k]);
                        let c2 = <$ty>::splat(values[k + 1]);
                        let (x1, _x2) = c1.interleave(c2);
                        let y1 = x1.utils_next_down();
                        for i in 0..<$ty>::LANES {
                            assert_eq!(y1.extract(i), x1.extract(i).utils_next_down());
                        }
                    }
                }
            )*
        }
    }

    simd_utils_next_down_tests! {
        test_utils_next_down_f32x2: (f32x2, f32),
        test_utils_next_down_f32x4: (f32x4, f32),
        test_utils_next_down_f32x8: (f32x8, f32),
        test_utils_next_down_f32x16: (f32x16, f32),
        test_utils_next_down_f64x2: (f64x2, f64),
        test_utils_next_down_f64x4: (f64x4, f64),
        test_utils_next_down_f64x8: (f64x8, f64),
    }

    macro_rules! compute_scale_scalar_tests {
        ($($fname:ident: $ty:ident,)*) => {
            $(
                #[test]
                fn $fname() {
                    // For each `(low, high)` pair in `v`, compute `scale` and
                    // verify that
                    //     scale * max_rand + low < high
                    // and
                    //     next_up(scale) * max_rand + low >= high
                    let eps = $ty::EPSILON;
                    let v = [
                        (0.0 as $ty, 100.0 as $ty),
                        (-0.125 as $ty, 0.0 as $ty),
                        (0.0 as $ty, 0.125 as $ty),
                        (-1.5 as $ty, -0.0 as $ty),
                        (-0.0 as $ty, 1.5 as $ty),
                        (-1.0 as $ty, -0.875 as $ty),
                        (-1e35 as $ty, -1e25 as $ty),
                        (1e-35 as $ty, 1e-25 as $ty),
                        (-1e35 as $ty, 1e35 as $ty),
                        // This one is mainly for f64, because these particular
                        // numbers provided an example where the initial calculation
                        // of `scale` is too small:
                        (11.0 as $ty, 1001.000000005 as $ty),
                        // Very small intervals--the difference `high - low` is
                        // a small to moderate multiple of the type's EPSILON.
                        (1.0 as $ty - (11.5  as $ty) * eps, 1.0  as $ty - (0.5 as $ty) * eps),
                        (1.0 as $ty - (196389.0 as $ty) * eps / (2.0 as $ty), 1.0 as $ty),
                        (1.0 as $ty, 1.0 as $ty + (1.0 as $ty) * eps),
                        (1.0 as $ty, 1.0 as $ty + (2.0 as $ty) * eps),
                        (1.0 as $ty - eps, 1.0 as $ty),
                        (1.0 as $ty - eps, 1.0 as $ty + (2.0 as $ty) * eps),
                        (-1.0 as $ty, -1.0 as $ty + (2.0 as $ty) * eps),
                        (-2.0 as $ty, -2.0 as $ty + (17.0 as $ty) * eps),
                        (-11.0 as $ty, -11.0 as $ty + (68.0 as $ty) *eps),
                        // Ridiculously small intervals: `low` and `high` are subnormal.
                        (-$ty::from_bits(3), $ty::from_bits(8)),
                        (-$ty::from_bits(5), -$ty::from_bits(1)),
                        // `high - low` is a significant fraction of the type's MAX.
                        ((0.5 as $ty) * $ty::MIN, (0.25 as $ty) * $ty::MAX),
                        ((0.25 as $ty) * $ty::MIN, (0.5 as $ty) * $ty::MAX),
                        ((0.5 as $ty) * $ty::MIN, (0.4999995 as $ty) * $ty::MAX),
                        ((0.75 as $ty) * $ty::MIN, 0.0 as $ty),
                        (0.0 as $ty, (0.75 as $ty) * $ty::MAX),
                    ];
                    let max_rand = 1.0 as $ty - eps;
                    for (low, high) in v {
                        let scale = <$ty>::compute_scale(low, high);
                        assert!(scale > 0.0 as $ty);
                        assert!(scale * max_rand + low < high);
                        // let next_scale = scale.next_up();
                        let next_scale = <$ty>::from_bits(scale.to_bits() + 1);
                        assert!(next_scale * max_rand + low >= high);
                    }
                }
            )*
        }
    }

    compute_scale_scalar_tests! {
        test_compute_scale_scalar_f32: f32,
        test_compute_scale_scalar_f64: f64,
    }

    macro_rules! compute_scale_simd_tests {
        ($($fname:ident: ($ty:ty, $f_scalar:ident),)*) => {
            $(
                #[test]
                #[cfg(feature = "simd_support")]
                fn $fname() {
                    let low_vals = [0.0 as $f_scalar, -1e35 as $f_scalar, 1e-7 as $f_scalar, -13.25 as $f_scalar];
                    let high_vals = [1.5 as $f_scalar, 3e35 as $f_scalar, 1.125 as $f_scalar, -0.0 as $f_scalar];
                    // Test that the vector version gives the same results as
                    // the scalar version.  Create test vectors that use two
                    // values at a time from arrays `low_vals` and `high_vals`.
                    for k in 0..(low_vals.len() - 1) {
                        let c1 = <$ty>::splat(low_vals[k]);
                        let c2 = <$ty>::splat(low_vals[k + 1]);
                        let (low, _) = c1.interleave(c2);
                        let c1 = <$ty>::splat(high_vals[k]);
                        let c2 = <$ty>::splat(high_vals[k + 1]);
                        let (high, _) = c1.interleave(c2);
                        let scale = <$ty>::compute_scale(low, high);
                        for i in 0..<$ty>::LANES {
                            assert_eq!(scale.extract(i), <$f_scalar>::compute_scale(low.extract(i), high.extract(i)));
                        }
                    }
                }
            )*
        }
    }

    compute_scale_simd_tests! {
        test_compute_scale_f32x2: (f32x2, f32),
        test_compute_scale_f32x4: (f32x4, f32),
        test_compute_scale_f32x8: (f32x8, f32),
        test_compute_scale_f32x16: (f32x16, f32),
        test_compute_scale_f64x2: (f64x2, f64),
        test_compute_scale_f64x4: (f64x4, f64),
        test_compute_scale_f64x8: (f64x8, f64),
    }
}
