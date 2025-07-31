use std::cmp::Ordering;

use crate::utils::errors::{AtlasError, Result};

use super::traits::Interpolate;

/// # Log-Linear Interpolator
/// Log-linear interpolator.
///
#[derive(Clone)]
pub struct LogLinearInterpolator {}

impl Interpolate for LogLinearInterpolator {
    fn interpolate(
        x: f64,
        x_: &Vec<f64>,
        y_: &Vec<f64>,
        enable_extrapolation: bool,
    ) -> Result<f64> {
        let index =
            match x_.binary_search_by(|&probe| probe.partial_cmp(&x).unwrap_or(Ordering::Less)) {
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
            0 => Ok(y_[0] * (y_[1] / y_[0]).powf((x - x_[0]) / (x_[1] - x_[0]))),
            idx if idx == x_.len() => {
                Ok(
                    y_[idx - 1]
                        * (y_[idx - 1] / y_[idx - 2])
                            .powf((x - x_[idx - 1]) / (x_[idx - 1] - x_[idx - 2]))
                )
            }
            _ => {
                Ok(
                    y_[index - 1]
                        * (y_[index] / y_[index - 1])
                            .powf((x - x_[index - 1]) / (x_[index] - x_[index - 1]))
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loglinear_interpolation() {
        let x = 0.5;
        let x_ = vec![0.0, 1.0];
        let y_ = vec![0.1, 1.0]; // Change from 0.0 to 0.1
        let y = LogLinearInterpolator::interpolate(x, &x_, &y_, true).unwrap();
        // Adjust the expected value accordingly
        assert!((y - 0.31622776601683794).abs() < 1e-10);
    }

    #[test]
    fn test_loglinear_interpolation_with_scale() {
        let scale = 2.0;
        let x = 0.5;
        let x_ = vec![0.0, 1.0];
        let y_ = vec![0.1 * scale, 1.0 * scale]; // Change from 0.0 to 0.1
        let y = LogLinearInterpolator::interpolate(x, &x_, &y_, true).unwrap();
        assert!((y - 0.31622776601683794 * scale).abs() < 1e-10);
    }

    #[test]
    fn test_loglinear_interpolation_with_scale_2() {
        let scale = 2.0;
        let x = 0.5;

        let x_ = vec![1.0, 2.0];

        let y_ = vec![0.1, 1.0];
        let y = LogLinearInterpolator::interpolate(x, &x_, &y_, true).unwrap();

        let y_ = vec![0.1 * scale, 1.0 * scale];
        let y_scale = LogLinearInterpolator::interpolate(x, &x_, &y_, true).unwrap();

        assert!((y * scale - y_scale).abs() < 1e-10);
    }

    #[test]
    fn test_loglinear_interpolation_with_scale_3() {
        let scale = 2.0;
        let x = 2.5;

        let x_ = vec![1.0, 2.0];

        let y_ = vec![0.1, 1.0];
        let y = LogLinearInterpolator::interpolate(x, &x_, &y_, true).unwrap();

        let y_ = vec![0.1 * scale, 1.0 * scale];
        let y_scale = LogLinearInterpolator::interpolate(x, &x_, &y_, true).unwrap();

        assert!((y * scale - y_scale).abs() < 1e-10);
    }

    #[test]
    fn test_loglinear_interpolation_at_bounds() {
        let x_ = vec![0.0, 1.0];
        let y_ = vec![0.1, 1.0];
        let y_start = LogLinearInterpolator::interpolate(0.0, &x_, &y_, true).unwrap();
        let y_end = LogLinearInterpolator::interpolate(1.0, &x_, &y_, true).unwrap();
        assert!((y_start - 0.1).abs() < 1e-10);
        assert!((y_end - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_loglinear_interpolation_extrapolation_disabled() {
        let x_ = vec![0.0, 1.0];
        let y_ = vec![0.1, 1.0];
        let result = LogLinearInterpolator::interpolate(-0.5, &x_, &y_, false);
        assert!(result.is_err());
        let result = LogLinearInterpolator::interpolate(1.5, &x_, &y_, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_loglinear_interpolation_extrapolation_enabled() {
        let x_ = vec![0.0, 1.0];
        let y_ = vec![0.1, 1.0];
        let y_left = LogLinearInterpolator::interpolate(-0.5, &x_, &y_, true).unwrap();
        let y_right = LogLinearInterpolator::interpolate(1.5, &x_, &y_, true).unwrap();
        // Just check that it returns a value and is positive
        assert!(y_left > 0.0);
        assert!(y_right > 0.0);
    }

    #[test]
    fn test_loglinear_interpolation_multiple_points() {
        let x_ = vec![0.0, 1.0, 2.0];
        let y_ = vec![0.1, 1.0, 10.0];
        let y_mid = LogLinearInterpolator::interpolate(1.5, &x_, &y_, true).unwrap();
        // Should be between y_[1] and y_[2]
        assert!(y_mid > 1.0 && y_mid < 10.0);
    }

    #[test]
    fn test_loglinear_interpolation_monotonicity() {
        let x_ = vec![0.0, 1.0, 2.0];
        let y_ = vec![0.1, 1.0, 10.0];
        let y1 = LogLinearInterpolator::interpolate(0.5, &x_, &y_, true).unwrap();
        let y2 = LogLinearInterpolator::interpolate(1.5, &x_, &y_, true).unwrap();
        assert!(y1 < y2);
    }

    #[test]
    fn test_loglinear_interpolation_discount_factors() {
        let scale = 2.0;
        let x1 = 186.0;
        let x2 = 276.0;
        let x_ = vec![x1, x2];
        let y1 = 0.976221377455272;
        let y2 = 0.965549861242730;
        let y_ = vec![y1, y2];
        let y = LogLinearInterpolator::interpolate(189.0, &x_, &y_, true).unwrap();
        assert!((y - 0.975863767225414) * scale < 1e-10);
    }
   
    #[test]
    fn test_loglinear_left_extrapolation_discount_factors() {
        let scale = 2.0;
        let x1 = 186.0;
        let x2 = 276.0;
        let x_ = vec![x1, x2];
        let y1 = 0.976221377455272;
        let y2 = 0.965549861242730;
        let y_ = vec![y1, y2];
        let y = LogLinearInterpolator::interpolate(100.0, &x_, &y_, true).unwrap();
        assert!((y - 0.986528784052285) * scale < 1e-10);
    }
 
    #[test]
    fn test_loglinear_right_extrapolation_discount_factors() {
        let scale = 2.0;
        let x1 = 186.0;
        let x2 = 276.0;
        let x_ = vec![x1, x2];
        let y1 = 0.976221377455272;
        let y2 = 0.965549861242730;
        let y_ = vec![y1, y2];
        let y = LogLinearInterpolator::interpolate(369.0, &x_, &y_, true).unwrap();
        assert!((y - 0.954645165621794) * scale < 1e-10);
    }
 
    
}
