use serde::{Deserialize, Serialize};

use super::{linear::LinearInterpolator, loglinear::LogLinearInterpolator, traits::Interpolate};

use crate::utils::errors::Result;

/// # Interpolator
/// Enum that represents the type of interpolation.
///
/// ## Example
/// ```
/// use rustatlas::prelude::*;
/// let x = 1.0;
/// let x_ = vec![0.0, 1.0, 2.0];
/// let y_ = vec![0.0, 1.0, 4.0];
/// let interpolator = Interpolator::Linear;
/// let y = interpolator.interpolate(x, &x_, &y_, true);
/// assert_eq!(y, 1.0);
/// ```
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum Interpolator {
    Linear,
    LogLinear,
}

impl Interpolator {
    pub fn interpolate(
        &self,
        x: f64,
        x_: &Vec<f64>,
        y_: &Vec<f64>,
        enable_extrapolation: bool,
    ) -> Result<f64> {
        match self {
            Interpolator::Linear => {
                LinearInterpolator::interpolate(x, x_, y_, enable_extrapolation)
            }
            Interpolator::LogLinear => {
                LogLinearInterpolator::interpolate(x, x_, y_, enable_extrapolation)
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_interpolation_basic() {
        let x = 1.0;
        let x_ = vec![0.0, 1.0, 2.0];
        let y_ = vec![0.0, 1.0, 4.0];
        let interpolator = Interpolator::Linear;
        let y = interpolator.interpolate(x, &x_, &y_, true).unwrap();
        assert_eq!(y, 1.0);
    }

    #[test]
    fn test_linear_interpolation_extrapolation() {
        let x = 3.0;
        let x_ = vec![0.0, 1.0, 2.0];
        let y_ = vec![0.0, 1.0, 4.0];
        let interpolator = Interpolator::Linear;
        let y = interpolator.interpolate(x, &x_, &y_, true).unwrap();
        assert_eq!(y, 7.0);
    }

    #[test]
    fn test_loglinear_interpolation_basic() {
        let x = 1.0;
        let x_ = vec![1.0, 2.0, 4.0];
        let y_ = vec![2.0, 4.0, 8.0];
        let interpolator = Interpolator::LogLinear;
        let y = interpolator.interpolate(x, &x_, &y_, true).unwrap();
        assert!((y - 2.0).abs() < 1e-8);
    }

    #[test]
    fn test_linear_interpolation_no_extrapolation() {
        let x = 3.0;
        let x_ = vec![0.0, 1.0, 2.0];
        let y_ = vec![0.0, 1.0, 4.0];
        let interpolator = Interpolator::Linear;
        let result = interpolator.interpolate(x, &x_, &y_, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_loglinear_interpolation_no_extrapolation() {
        let x = 8.0;
        let x_ = vec![1.0, 2.0, 4.0];
        let y_ = vec![2.0, 4.0, 8.0];
        let interpolator = Interpolator::LogLinear;
        let result = interpolator.interpolate(x, &x_, &y_, false);
        assert!(result.is_err());
    }

}