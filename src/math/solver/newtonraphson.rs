use crate::{
    math::solver::traits::{Gradient, Hessian, SolverError},
    utils::errors::{AtlasError, Result},
};
use nalgebra::DVector;

#[derive(Debug)]
pub struct NewtonRaphsonResult {
    pub solution: DVector<f64>,
    pub iterations: usize,
    pub final_gradient_norm: f64,
    pub converged: bool,
}

pub struct NewtonRaphson<P>
where
    P: Gradient + Hessian,
{
    problem: P,
    max_iterations: usize,
    tolerance: f64,
    initial_guess: DVector<f64>,
}

impl<P> NewtonRaphson<P>
where
    P: Gradient + Hessian,
{
    pub fn new(problem: P, initial_guess: DVector<f64>) -> Self {
        Self {
            problem,
            max_iterations: 100,
            tolerance: 1e-8,
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

    pub fn solve(&self) -> Result<NewtonRaphsonResult> {
        let mut x = self.initial_guess.clone();
        let mut converged = false;
        let mut iterations = 0;
        let mut final_gradient_norm = 0.0;

        for i in 0..self.max_iterations {
            iterations = i + 1;
            let grad = self.problem.gradient(&x)?;
            final_gradient_norm = grad.norm();
            if final_gradient_norm < self.tolerance {
                converged = true;
                break;
            }

            let hess = self.problem.hessian(&x)?;
            let delta = match hess.lu().solve(&(-grad)) {
                Some(sol) => sol,
                None => {
                    return Err(AtlasError::SolverError(SolverError::SingularHessian(
                        "Hessian matrix is singular".to_string(),
                    )));
                }
            };

            x += delta.clone();

            if delta.norm() < self.tolerance {
                converged = true;
                break;
            }
        }

        if !converged {
            return Err(AtlasError::SolverError(SolverError::MaxIterationsReached(
                "Maximum iterations reached without convergence".to_string(),
            )));
        }

        Ok(NewtonRaphsonResult {
            solution: x,
            iterations,
            final_gradient_norm,
            converged,
        })
    }
}

#[cfg(test)]
mod tests {
    use nalgebra::DMatrix;
    use super::*;

    struct QuadraticProblem;

    impl Gradient for QuadraticProblem {
        fn gradient(&self, x: &DVector<f64>) -> Result<DVector<f64>> {
            // f(x) = 0.5 * x^T x, grad = x
            Ok(x.clone())
        }
    }

    impl Hessian for QuadraticProblem {
        fn hessian(&self, _x: &DVector<f64>) -> Result<DMatrix<f64>> {
            // Hessian is identity
            Ok(DMatrix::identity(1, 1))
        }
    }

    #[test]
    fn test_newtonraphson_quadratic_converges() {
        let problem = QuadraticProblem;
        let initial_guess = DVector::from_vec(vec![10.0]);
        let solver = NewtonRaphson::new(problem, initial_guess)
            .with_max_iterations(50)
            .with_tolerance(1e-10);

        let result = solver.solve().unwrap();
        assert!(result.converged);
        assert!(result.final_gradient_norm < 1e-10);
        assert!((result.solution[0]).abs() < 1e-10);
        assert!(result.iterations < 10);
    }

    struct SingularProblem;

    impl Gradient for SingularProblem {
        fn gradient(&self, x: &DVector<f64>) -> Result<DVector<f64>> {
            Ok(DVector::from_vec(vec![x[0]]))
        }
    }

    impl Hessian for SingularProblem {
        fn hessian(&self, _x: &DVector<f64>) -> Result<DMatrix<f64>> {
            // Singular Hessian (zero matrix)
            Ok(DMatrix::zeros(1, 1))
        }
    }

    #[test]
    fn test_newtonraphson_singular_hessian_error() {
        let problem = SingularProblem;
        let initial_guess = DVector::from_vec(vec![1.0]);
        let solver = NewtonRaphson::new(problem, initial_guess);

        let result = solver.solve();
        assert!(result.is_err());
        if let Err(AtlasError::SolverError(SolverError::SingularHessian(_))) = result {
            // expected
        } else {
            panic!("Expected SingularHessian error");
        }
    }

    struct NoConvergenceProblem;

    impl Gradient for NoConvergenceProblem {
        fn gradient(&self, _x: &DVector<f64>) -> Result<DVector<f64>> {
            Ok(DVector::from_vec(vec![1.0]))
        }
    }

    impl Hessian for NoConvergenceProblem {
        fn hessian(&self, _x: &DVector<f64>) -> Result<DMatrix<f64>> {
            Ok(DMatrix::identity(1, 1))
        }
    }

    #[test]
    fn test_newtonraphson_max_iterations_reached() {
        let problem = NoConvergenceProblem;
        let initial_guess = DVector::from_vec(vec![0.0]);
        let solver = NewtonRaphson::new(problem, initial_guess)
            .with_max_iterations(5)
            .with_tolerance(1e-12);

        let result = solver.solve();
        assert!(result.is_err());
        if let Err(AtlasError::SolverError(SolverError::MaxIterationsReached(_))) = result {
            // expected
        } else {
            panic!("Expected MaxIterationsReached error");
        }
    }
}
