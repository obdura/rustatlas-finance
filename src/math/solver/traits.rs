use nalgebra::{DMatrix, DVector};
use thiserror::Error;
use crate::utils::errors::Result;

pub trait Residual {
    fn residual(&self, x: &DVector<f64>) -> Result<DVector<f64>>;
}

pub trait Gradient {
    fn gradient(&self, x: &DVector<f64>) -> Result<DVector<f64>>;
}

pub trait Jacobian {
    fn jacobian(&self, x: &DVector<f64>) -> Result<DMatrix<f64>>;   
}

pub trait Hessian {
    fn hessian(&self, x: &DVector<f64>) -> Result<DMatrix<f64>>;
}

pub trait CostFunction {
    fn cost(&self, param: &f64) -> Result<f64>;
}

#[derive(Debug, Error)]
pub enum SolverError {
    #[error("Solver error: {0}")]
    SingularMatrix(String),
    #[error("Solver error: {0}")]
    SingularHessian(String),
    #[error("Solver error: {0}")]
    MaxIterationsReached(String),
    #[error("Solver error: {0}")]
    BrentRootError(String),
}

