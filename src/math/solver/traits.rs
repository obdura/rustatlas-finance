use nalgebra::{DMatrix, DVector};
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
