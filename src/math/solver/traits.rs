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
    type Param;
    type Output;
    fn cost(&self, param: &Self::Param) -> Result<Self::Output>;
}

#[derive(Debug, Error)]
pub enum SolverError {
    #[error("Solver error: {0}")]
    SingularMatrix(String),
    #[error("Solver error: {0}")]
    SingularHessian(String),
    #[error("Solver error: {0}")]
    MaxIterationsReached(String),
}

