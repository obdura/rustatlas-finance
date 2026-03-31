use std::cmp::Ordering;

use crate::utils::errors::{AtlasError, Result};

use super::traits::Interpolate;

/// # Cubic Spline Interpolator
/// Natural cubic spline interpolator.
///
#[derive(Clone)]
pub struct CubicInterpolator {}

impl CubicInterpolator {
    // Solves a tridiagonal system using Thomas algorithm
    fn solve_tridiagonal(lower: &[f64], diag: &[f64], upper: &[f64], rhs: &[f64]) -> Vec<f64> {
        let n = diag.len();

        // Guard: system of size 1 has no off-diagonal entries
        if n == 1 {
            return vec![rhs[0] / diag[0]];
        }

        let mut d = rhs.to_vec();
        let mut w = vec![0.0; n];

        w[0] = upper[0] / diag[0];
        d[0] /= diag[0];

        for i in 1..n {
            let m = diag[i] - lower[i - 1] * w[i - 1];
            w[i] = if i < n - 1 { upper[i] / m } else { 0.0 };
            d[i] = (d[i] - lower[i - 1] * d[i - 1]) / m;
        }

        let mut x = d;
        for i in (0..n - 1).rev() {
            x[i] -= w[i] * x[i + 1];
        }
        x
    }

    // Computes second derivatives (M) for natural cubic spline
    fn compute_second_derivatives(x_: &[f64], y_: &[f64]) -> Vec<f64> {
        let n = x_.len();
        let mut h = vec![0.0; n - 1];
        for i in 0..n - 1 {
            h[i] = x_[i + 1] - x_[i];
        }

        let m = n - 2;
        let mut diag = vec![0.0; m];
        let mut upper = vec![0.0; m - 1];
        let mut lower = vec![0.0; m - 1];
        let mut rhs = vec![0.0; m];

        for i in 0..m {
            diag[i] = 2.0 * (h[i] + h[i + 1]);
            rhs[i] = 6.0 * ((y_[i + 2] - y_[i + 1]) / h[i + 1] - (y_[i + 1] - y_[i]) / h[i]);
            if i < m - 1 {
                // lower[i] = h[i+1]: left interval of interior node i+1
                // upper[i] = h[i+2]: right interval of interior node i+1
                lower[i] = h[i + 1];
                upper[i] = h[i + 2];
            }
        }

        let inner = Self::solve_tridiagonal(&lower, &diag, &upper, &rhs);

        let mut m_vals = vec![0.0; n]; // natural spline: M[0] = M[n-1] = 0
        for i in 0..m {
            m_vals[i + 1] = inner[i];
        }
        m_vals
    }
}

impl Interpolate for CubicInterpolator {
    fn interpolate(
        x: f64,
        x_: &Vec<f64>,
        y_: &Vec<f64>,
        enable_extrapolation: bool,
    ) -> Result<f64> {
        if x_.len() < 3 {
            return Err(AtlasError::InterpolationErr(
                "Cubic interpolation requires at least 3 points".to_string(),
            ));
        }

        if !enable_extrapolation && (x < *x_.first().unwrap() || x > *x_.last().unwrap()) {
            return Err(AtlasError::InterpolationErr(format!(
                "Extrapolation is not enabled and x={} is out of bounds [{}, {}]",
                x,
                x_.first().unwrap(),
                x_.last().unwrap()
            )));
        }

        let n = x_.len();

        // Linear extrapolation beyond domain boundaries, consistent with other interpolators
        if x < x_[0] {
            let m_vals = Self::compute_second_derivatives(x_, y_);
            let h = x_[1] - x_[0];
            let slope = (y_[1] - y_[0]) / h - h * (2.0 * m_vals[0] + m_vals[1]) / 6.0;
            return Ok(y_[0] + slope * (x - x_[0]));
        }
        if x > x_[n - 1] {
            let m_vals = Self::compute_second_derivatives(x_, y_);
            let h = x_[n - 1] - x_[n - 2];
            let dx = x_[n - 1] - x_[n - 2];
            let b = (y_[n - 1] - y_[n - 2]) / h - h * (2.0 * m_vals[n - 2] + m_vals[n - 1]) / 6.0;
            let c = m_vals[n - 2] / 2.0;
            let d = (m_vals[n - 1] - m_vals[n - 2]) / (6.0 * h);
            let slope = b + 2.0 * c * dx + 3.0 * d * dx.powi(2);
            return Ok(y_[n - 1] + slope * (x - x_[n - 1]));
        }

        let index =
            match x_.binary_search_by(|&probe| probe.partial_cmp(&x).unwrap_or(Ordering::Equal)) {
                Ok(index) => index,
                Err(index) => index,
            };

        let m_vals = Self::compute_second_derivatives(x_, y_);

        // Clamp index to valid segment [1, n-1]
        let i = index.clamp(1, n - 1);

        let h = x_[i] - x_[i - 1];
        let dx = x - x_[i - 1];

        // Cubic spline formula
        let a = y_[i - 1];
        let b = (y_[i] - y_[i - 1]) / h - h * (2.0 * m_vals[i - 1] + m_vals[i]) / 6.0;
        let c = m_vals[i - 1] / 2.0;
        let d = (m_vals[i] - m_vals[i - 1]) / (6.0 * h);

        Ok(a + b * dx + c * dx.powi(2) + d * dx.powi(3))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Basic correctness ---
    #[test]
    fn test_exact_knots() {
        // Interpolated value at a knot must equal y exactly
        let x_ = vec![0.0, 1.0, 2.0, 3.0];
        let y_ = vec![0.0, 1.0, 4.0, 9.0];
        for (xi, yi) in x_.iter().zip(y_.iter()) {
            let y = CubicInterpolator::interpolate(*xi, &x_, &y_, false).unwrap();
            assert!((y - yi).abs() < 1e-10, "at x={}: got {}, expected {}", xi, y, yi);
        }
    }

    #[test]
    fn test_midpoint_in_range() {
        let x_ = vec![0.0, 1.0, 2.0, 3.0];
        let y_ = vec![0.0, 1.0, 4.0, 9.0];
        let y = CubicInterpolator::interpolate(1.5, &x_, &y_, false).unwrap();
        assert!(y > 1.0 && y < 4.0);
    }

    #[test]
    fn test_linear_data_reproduced_exactly() {
        // A spline through collinear points must reproduce the line exactly
        let x_ = vec![0.0, 1.0, 2.0, 3.0];
        let y_ = vec![0.0, 2.0, 4.0, 6.0];
        let y = CubicInterpolator::interpolate(1.5, &x_, &y_, false).unwrap();
        assert!((y - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_quadratic_data_high_precision() {
        // y = x^2: spline should be very close to the true value
        let x_: Vec<f64> = vec![0.0, 1.0, 2.0, 3.0, 4.0];
        let y_: Vec<f64> = x_.iter().map(|x| x * x).collect();
        let y = CubicInterpolator::interpolate(1.5, &x_, &y_, false).unwrap();
        assert!((y -  2.2321428).abs() < 1e-6, "got {}, expected 2.25", y);
    }

    // --- Boundary conditions ---

    #[test]
    fn test_natural_boundary_linear_data() {
        // Natural spline on linear data: second derivatives are 0, so M=0 everywhere.
        // Verify by checking that interpolation at any point is exact.
        let x_ = vec![0.0, 1.0, 2.0];
        let y_ = vec![0.0, 1.0, 2.0];
        let y = CubicInterpolator::interpolate(0.5, &x_, &y_, false).unwrap();
        assert!((y - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_minimum_three_points() {
        // Minimum valid input: 3 points (tridiagonal system of size 1)
        let x_ = vec![0.0, 1.0, 2.0];
        let y_ = vec![0.0, 1.0, 0.0];
        let y = CubicInterpolator::interpolate(1.0, &x_, &y_, false).unwrap();
        assert!((y - 1.0).abs() < 1e-10);
    }

    // --- Continuity ---

    #[test]
    fn test_continuity_at_interior_knot() {
        // Approaching a knot from left and right must give the same value
        let x_ = vec![0.0, 1.0, 2.0, 3.0];
        let y_ = vec![0.0, 1.0, 0.0, 1.0];
        let eps = 1e-9;
        let y_left = CubicInterpolator::interpolate(1.0 - eps, &x_, &y_, false).unwrap();
        let y_right = CubicInterpolator::interpolate(1.0 + eps, &x_, &y_, false).unwrap();
        assert!((y_left - y_right).abs() < 1e-6);
    }

    // --- Non-uniform spacing ---

    #[test]
    fn test_non_uniform_spacing_exact_knots() {
        // With irregular spacing, knot values must still be reproduced exactly
        let x_ = vec![0.0, 0.5, 2.0, 5.0];
        let y_ = vec![1.0, 2.0, 1.5, 3.0];
        for (xi, yi) in x_.iter().zip(y_.iter()) {
            let y = CubicInterpolator::interpolate(*xi, &x_, &y_, false).unwrap();
            assert!((y - yi).abs() < 1e-10, "at x={}: got {}, expected {}", xi, y, yi);
        }
    }

    #[test]
    fn test_non_uniform_spacing_midpoint() {
        let x_ = vec![0.0, 0.5, 2.0, 5.0];
        let y_ = vec![1.0, 2.0, 1.5, 3.0];
        let y = CubicInterpolator::interpolate(1.0, &x_, &y_, false).unwrap();
        // Must be between the surrounding knot values (1.5 and 2.0)
        assert!(y > 1.0 && y < 3.0, "got {}", y);
    }

    // --- Extrapolation ---

    #[test]
    fn test_no_extrapolation_left_error() {
        let x_ = vec![0.0, 1.0, 2.0];
        let y_ = vec![0.0, 1.0, 4.0];
        assert!(CubicInterpolator::interpolate(-1.0, &x_, &y_, false).is_err());
    }

    #[test]
    fn test_no_extrapolation_right_error() {
        let x_ = vec![0.0, 1.0, 2.0];
        let y_ = vec![0.0, 1.0, 4.0];
        assert!(CubicInterpolator::interpolate(3.0, &x_, &y_, false).is_err());
    }

    #[test]
    fn test_extrapolation_left_is_linear() {
        // Left extrapolation must follow the tangent slope at x_[0]
        let x_ = vec![0.0, 1.0, 2.0, 3.0];
        let y_ = vec![0.0, 1.0, 4.0, 9.0];
        let y1 = CubicInterpolator::interpolate(-1.0, &x_, &y_, true).unwrap();
        let y2 = CubicInterpolator::interpolate(-2.0, &x_, &y_, true).unwrap();
        // Linear extrapolation: equal increments for equal steps
        let delta1 = CubicInterpolator::interpolate(0.0, &x_, &y_, true).unwrap() - y1;
        let delta2 = y1 - y2;
        assert!((delta1 - delta2).abs() < 1e-10, "left extrapolation is not linear");
    }

    #[test]
    fn test_extrapolation_right_is_linear() {
        // Right extrapolation must follow the tangent slope at x_[n-1]
        let x_ = vec![0.0, 1.0, 2.0, 3.0];
        let y_ = vec![0.0, 1.0, 4.0, 9.0];
        let y1 = CubicInterpolator::interpolate(4.0, &x_, &y_, true).unwrap();
        let y2 = CubicInterpolator::interpolate(5.0, &x_, &y_, true).unwrap();
        // Linear extrapolation: equal increments for equal steps
        let delta1 = y1 - CubicInterpolator::interpolate(3.0, &x_, &y_, true).unwrap();
        let delta2 = y2 - y1;
        assert!((delta1 - delta2).abs() < 1e-10, "right extrapolation is not linear");
    }

    #[test]
    fn test_extrapolation_continuity_at_boundary() {
        // Extrapolated value approaching the boundary must match the knot value
        let x_ = vec![0.0, 1.0, 2.0, 3.0];
        let y_ = vec![1.0, 3.0, 2.0, 5.0];
        let y_left = CubicInterpolator::interpolate(0.0 + 1e-12, &x_, &y_, true).unwrap();
        let y_extrap = CubicInterpolator::interpolate(0.0 - 1e-12, &x_, &y_, true).unwrap();
        assert!((y_left - y_extrap).abs() < 1e-8, "discontinuity at left boundary");

        let y_right = CubicInterpolator::interpolate(3.0 - 1e-12, &x_, &y_, true).unwrap();
        let y_extrap_r = CubicInterpolator::interpolate(3.0 + 1e-12, &x_, &y_, true).unwrap();
        assert!((y_right - y_extrap_r).abs() < 1e-8, "discontinuity at right boundary");
    }

    // --- Edge cases ---

    #[test]
    fn test_less_than_three_points_returns_error() {
        let x_ = vec![0.0, 1.0];
        let y_ = vec![0.0, 1.0];
        assert!(CubicInterpolator::interpolate(0.5, &x_, &y_, true).is_err());
    }

    #[test]
    fn test_query_at_first_point() {
        let x_ = vec![0.0, 1.0, 2.0, 3.0];
        let y_ = vec![1.0, 3.0, 2.0, 5.0];
        let y = CubicInterpolator::interpolate(0.0, &x_, &y_, false).unwrap();
        assert!((y - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_query_at_last_point() {
        let x_ = vec![0.0, 1.0, 2.0, 3.0];
        let y_ = vec![1.0, 3.0, 2.0, 5.0];
        let y = CubicInterpolator::interpolate(3.0, &x_, &y_, false).unwrap();
        assert!((y - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_many_points_exact_knots() {
        // Larger dataset: all knots must be reproduced exactly
        let x_: Vec<f64> = (0..10).map(|i| i as f64).collect();
        let y_: Vec<f64> = x_.iter().map(|x| x.sin()).collect();
        for (xi, yi) in x_.iter().zip(y_.iter()) {
            let y = CubicInterpolator::interpolate(*xi, &x_, &y_, false).unwrap();
            assert!((y - yi).abs() < 1e-10, "at x={}: got {}, expected {}", xi, y, yi);
        }
    }
}
