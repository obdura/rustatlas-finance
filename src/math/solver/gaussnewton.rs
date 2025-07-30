use nalgebra::DVector;
use thiserror::Error;

use crate::{
    math::solver::traits::{Jacobian, Residual},
    utils::errors::{AtlasError, Result},
};

#[derive(Debug, Error)]
pub enum GaussNewtonError {
    #[error("Solver error: {0}")]
    SingularMatrix(String),
    #[error("Solver error: {0}")]
    MaxIterationsReached(String),
}

#[derive(Debug)]
pub struct GaussNewtonResult {
    pub solution: DVector<f64>,
    pub iterations: usize,
    pub final_residual_norm: f64,
    pub converged: bool,
}

pub struct GaussNewton<P>
where
    P: Residual + Jacobian,
{
    problem: P,
    max_iterations: usize,
    tolerance: f64,
    initial_guess: DVector<f64>, // <- nuevo campo
}

impl<P> GaussNewton<P>
where
    P: Residual + Jacobian,
{
    pub fn new(problem: P, initial_guess: DVector<f64>) -> Self {
        Self {
            problem,
            max_iterations: 100, // Default max iterations
            tolerance: 1e-8,     // Default tolerance
            initial_guess,
        }
    }

    pub fn with_max_iterations(mut self, max_iterations: usize) -> Self {
        self.max_iterations = max_iterations;
        self
    }

    pub fn with_tolerance(mut self, tolerance: f64) -> Self {
        self.tolerance = tolerance;
        self
    }

    pub fn solve(&self) -> Result<GaussNewtonResult> {
        let mut x = self.initial_guess.clone();
        let mut converged = false;
        let mut iterations = 0;
        let mut final_residual_norm = 0.0;

        for i in 0..self.max_iterations {
            iterations = i + 1;
            let r = self.problem.residual(&x)?;
            final_residual_norm = r.norm();
            if final_residual_norm < self.tolerance {
                converged = true;
                break;
            }

            let j = self.problem.jacobian(&x)?;
            let jt = j.transpose();
            let jtj = &jt * &j;
            let jtr = &jt * &r;

            let delta = match jtj.lu().solve(&(-jtr)) {
                Some(sol) => sol,
                None => {
                    return Err(AtlasError::SolverError(
                        GaussNewtonError::SingularMatrix("Jacobian is singular".to_string()),
                    ));
                }
            };

            x += delta.clone();

            if delta.norm() < self.tolerance {
                converged = true;
                break;
            }
        }

        if !converged {
            return Err(AtlasError::SolverError(
                GaussNewtonError::MaxIterationsReached("Maximum iterations reached without convergence".to_string()),
            ));
        }

        Ok(GaussNewtonResult {
            solution: x,
            iterations,
            final_residual_norm,
            converged,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use nalgebra::{DMatrix, DVector};

    struct SquareFit {
        data: DVector<f64>,
    }

    impl Residual for SquareFit {
        fn residual(&self, x: &DVector<f64>) -> Result<DVector<f64>> {
            Ok(x.map(|xi| xi * xi) - &self.data)
        }
    }

    impl Jacobian for SquareFit {
        fn jacobian(&self, x: &DVector<f64>) -> Result<DMatrix<f64>> {
            let n = x.len();
            let mut j = DMatrix::zeros(n, n);
            for i in 0..n {
                j[(i, i)] = 2.0 * x[i];
            }
            Ok(j)
        }
    }

    #[test]
    fn test_gauss_newton_non_converging_case() {
        // Provide a singular Jacobian (all zeros), should trigger SingularMatrix error
        struct SingularFit;
        impl Residual for SingularFit {
            fn residual(&self, _x: &DVector<f64>) -> Result<DVector<f64>> {
                Ok(DVector::from_vec(vec![1.0, 2.0]))
            }
        }
        impl Jacobian for SingularFit {
            fn jacobian(&self, _x: &DVector<f64>) -> Result<DMatrix<f64>> {
                Ok(DMatrix::zeros(2, 2))
            }
        }
        let problem = SingularFit;
        let initial_guess = DVector::from_vec(vec![0.0, 0.0]);
        let solver = GaussNewton::new(problem, initial_guess);
        let result = solver.solve();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AtlasError::SolverError(GaussNewtonError::SingularMatrix(_))
        ));
    }

    #[test]
    fn test_gauss_newton_max_iterations_reached() {
        // Use a problem that never converges within a low iteration limit
        struct SlowFit;
        impl Residual for SlowFit {
            fn residual(&self, _x: &DVector<f64>) -> Result<DVector<f64>> {
                Ok(DVector::from_vec(vec![100.0]))
            }
        }
        impl Jacobian for SlowFit {
            fn jacobian(&self, _x: &DVector<f64>) -> Result<DMatrix<f64>> {
                Ok(DMatrix::identity(1, 1))
            }
        }
        let problem = SlowFit;
        let initial_guess = DVector::from_vec(vec![0.0]);
        let solver = GaussNewton::new(problem, initial_guess).with_max_iterations(2);
        let result = solver.solve();
        assert!(result.is_err());
        let err = result.unwrap_err();
        if let AtlasError::SolverError(gauss_err) = err {
            assert!(matches!(gauss_err, GaussNewtonError::MaxIterationsReached(_)));
        } else {
            panic!("Expected AtlasError::SolverError");
        }
    }

    #[test]
    fn test_gauss_newton_already_converged_initial_guess() {
        // Initial guess is already the solution
        let data = DVector::from_vec(vec![1.0, 4.0, 9.0]);
        let problem = SquareFit { data: data.clone() };
        let initial_guess = DVector::from_vec(vec![1.0, 2.0, 3.0]);
        let solver = GaussNewton::new(problem, initial_guess);
        let result = solver.solve();
        let expected = DVector::from_vec(vec![1.0, 2.0, 3.0]);
        let result = result.unwrap();
        assert!(result.converged);
        for i in 0..expected.len() {
            assert_relative_eq!(result.solution[i], expected[i], epsilon = 1e-8);
        }
        assert!(result.final_residual_norm < solver.tolerance);
    }

    #[test]
    fn test_gauss_newton_large_problem() {
        // Test with a larger vector
        let n = 10;
        let data: Vec<f64> = (1..=n).map(|i| (i as f64).powi(2)).collect();
        let expected: Vec<f64> = (1..=n).map(|i| i as f64).collect();
        let problem = SquareFit {
            data: DVector::from_vec(data.clone()),
        };
        let initial_guess = DVector::from_element(n, 0.5);
        let solver = GaussNewton::new(problem, initial_guess)
            .with_max_iterations(200)
            .with_tolerance(1e-10);
        let result = solver.solve().unwrap();
        for i in 0..n {
            assert_relative_eq!(result.solution[i].abs(), expected[i], epsilon = 1e-6);
        }
    }

    #[test]
    fn test_gauss_newton_square_fit() {
        // Queremos que x converja a [2.0, 3.0, 4.0] dado que x^2 = [4.0, 9.0, 16.0]
        let data = DVector::from_vec(vec![4.0, 9.0, 16.0]);
        let problem = SquareFit { data };
        let initial_guess = DVector::from_vec(vec![1.0, 1.0, 1.0]);
        let solver = GaussNewton::new(problem, initial_guess);
        let result = solver.solve().unwrap();

        let expected = DVector::from_vec(vec![2.0, 3.0, 4.0]);
        for i in 0..expected.len() {
            // Usa tolerancia relativa para evitar problemas por flotantes
            assert_relative_eq!(result.solution[i].abs(), expected[i], epsilon = 1e-6);
        }
    }

    #[test]
    fn test_gauss_newton_zero_solution() {
        // x^2 = [0.0, 0.0, 0.0] => x = [0.0, 0.0, 0.0]
        let data = DVector::from_vec(vec![0.0, 0.0, 0.0]);
        let problem = SquareFit { data };
        let initial_guess = DVector::from_vec(vec![0.5, -0.5, 1.0]);
        let solver = GaussNewton::new(problem, initial_guess)
            .with_max_iterations(100)
            .with_tolerance(1e-15);
        let result = solver.solve().unwrap();

        let expected = DVector::from_vec(vec![0.0, 0.0, 0.0]);
        for i in 0..expected.len() {
            assert_relative_eq!(result.solution[i].abs(), expected[i], epsilon = 1e-6);
        }
    }

    #[test]
    fn test_gauss_newton_negative_solution() {
        // x^2 = [1.0, 4.0, 9.0] => x = [-1.0, -2.0, -3.0] (abs checked)
        let data = DVector::from_vec(vec![1.0, 4.0, 9.0]);
        let problem = SquareFit { data };
        let initial_guess = DVector::from_vec(vec![-0.5, -1.5, -2.5]);
        let solver = GaussNewton::new(problem, initial_guess);
        let result = solver.solve().unwrap();

        let expected = DVector::from_vec(vec![1.0, 2.0, 3.0]);
        for i in 0..expected.len() {
            assert_relative_eq!(result.solution[i].abs(), expected[i], epsilon = 1e-6);
        }
    }

    #[test]
    fn test_gauss_newton_custom_tolerance_and_iterations() {
        let data = DVector::from_vec(vec![25.0, 36.0]);
        let problem = SquareFit { data };
        let initial_guess = DVector::from_vec(vec![1.0, 1.0]);
        let solver = GaussNewton::new(problem, initial_guess)
            .with_max_iterations(200)
            .with_tolerance(1e-10);
        let result = solver.solve().unwrap();

        let expected = DVector::from_vec(vec![5.0, 6.0]);
        for i in 0..expected.len() {
            assert_relative_eq!(result.solution[i].abs(), expected[i], epsilon = 1e-8);
        }
    }
}
