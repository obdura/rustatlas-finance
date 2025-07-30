use std::cmp::Ordering;

use crate::utils::errors::{AtlasError, Result};

use super::traits::Interpolate;

/// # Linear Interpolator
/// Basic linear interpolator.
///
#[derive(Clone)]
pub struct LinearInterpolator {}

impl Interpolate for LinearInterpolator {
    fn interpolate(
        x: f64,
        x_: &Vec<f64>,
        y_: &Vec<f64>,
        enable_extrapolation: bool,
    ) -> Result<f64> {
        let index =
            match x_.binary_search_by(|&probe| probe.partial_cmp(&x).unwrap_or(Ordering::Equal)) {
                Ok(index) => index,
                Err(index) => index,
            };

        if !enable_extrapolation && (x < *x_.first().unwrap() || x > *x_.last().unwrap()) {
            return Err(AtlasError::InterpolationErr(format!(
                "Extrapolation is not enabled and x={} is out of bounds [{}, {}]",
                x,
                x_.first().unwrap(),
                x_.last().unwrap()
            )));
        }

        match index {
            0 => Ok(y_[0] + (x - x_[0]) * (y_[1] - y_[0]) / (x_[1] - x_[0])),
            index if index == x_.len() => {
                Ok(
                    y_[index - 1]
                        + (x - x_[index - 1]) * (y_[index - 1] - y_[index - 2])
                            / (x_[index - 1] - x_[index - 2])
                )
            }
            _ => {
                Ok(
                    y_[index - 1]
                        + (x - x_[index - 1]) * (y_[index] - y_[index - 1])
                            / (x_[index] - x_[index - 1])
                )
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::Interpolate;
    use super::LinearInterpolator;

    #[test]
    fn test_linear_interpolation() {
        let x = 0.5;
        let x_ = vec![0.0, 1.0];
        let y_ = vec![0.0, 1.0];
        let y = LinearInterpolator::interpolate(x, &x_, &y_, true).unwrap();
        assert_eq!(y, 0.5);
    }

    #[test]
    fn test_exact_point() {
        let x = 1.0;
        let x_ = vec![0.0, 1.0, 2.0];
        let y_ = vec![0.0, 2.0, 4.0];
        let y = LinearInterpolator::interpolate(x, &x_, &y_, true).unwrap();
        assert_eq!(y, 2.0);
    }

    #[test]
    fn test_extrapolation_left() {
        let x = -1.0;
        let x_ = vec![0.0, 1.0];
        let y_ = vec![0.0, 1.0];
        let y = LinearInterpolator::interpolate(x, &x_, &y_, true).unwrap();
        assert_eq!(y, -1.0);
    }

    #[test]
    fn test_extrapolation_right() {
        let x = 2.0;
        let x_ = vec![0.0, 1.0];
        let y_ = vec![0.0, 1.0];
        let y = LinearInterpolator::interpolate(x, &x_, &y_, true).unwrap();
        assert_eq!(y, 2.0);
    }

    #[test]
    fn test_no_extrapolation_left_error() {
        let x = -1.0;
        let x_ = vec![0.0, 1.0];
        let y_ = vec![0.0, 1.0];
        let result = LinearInterpolator::interpolate(x, &x_, &y_, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_no_extrapolation_right_error() {
        let x = 2.0;
        let x_ = vec![0.0, 1.0];
        let y_ = vec![0.0, 1.0];
        let result = LinearInterpolator::interpolate(x, &x_, &y_, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_points_interpolation() {
        let x = 1.5;
        let x_ = vec![0.0, 1.0, 2.0];
        let y_ = vec![0.0, 2.0, 4.0];
        let y = LinearInterpolator::interpolate(x, &x_, &y_, true).unwrap();
        assert_eq!(y, 3.0);
    }
}
