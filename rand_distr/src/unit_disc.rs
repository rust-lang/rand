// Copyright 2019 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use num_traits::Float;
use crate::{uniform::SampleUniform, Distribution, Uniform};
use rand::Rng;

/// Samples uniformly from the unit disc in two dimensions.
///
/// Implemented via rejection sampling.
///
///
/// # Example
///
/// ```
/// use rand_distr::{UnitDisc, Distribution};
///
/// let v: [f64; 2] = UnitDisc.sample(&mut rand::thread_rng());
/// println!("{:?} is from the unit Disc.", v)
/// ```
#[derive(Clone, Copy, Debug)]
pub struct UnitDisc;

impl<F: Float + SampleUniform> Distribution<[F; 2]> for UnitDisc {
    #[inline]
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> [F; 2] {
        let uniform = Uniform::new(F::from(-1.).unwrap(), F::from(1.).unwrap());
        let mut x1;
        let mut x2;
        loop {
            x1 = uniform.sample(rng);
            x2 = uniform.sample(rng);
            if x1 * x1 + x2 * x2 <= F::from(1.).unwrap() {
                break;
            }
        }
        [x1, x2]
    }
}

#[cfg(test)]
mod tests {
    use super::UnitDisc;
    use crate::Distribution;

    #[test]
    fn value_stability() {
        let mut rng = crate::test::rng(2);
        let expected = [
            [0.018035709265959987, -0.4348771383120438],
            [-0.07982762085055706, 0.7765329819820659],
            [0.21450745997299503, 0.7398636984333291],
        ];
        let samples: [[f64; 2]; 3] = [
            UnitDisc.sample(&mut rng),
            UnitDisc.sample(&mut rng),
            UnitDisc.sample(&mut rng),
        ];
        assert_eq!(samples, expected);
    }
}
