use crate::utils::errors::Result;

/// # Interpolation trait
/// A trait that defines the interpolation of a function.
/// 
/// * interpolate - Interpolates a function at a given point x.
/// * x - The point at which to interpolate the function.
/// * x_ - The x values of the function.
/// * y_ - The y values of the function.
/// * enable_extrapolation - A flag to enable extrapolation.
/// 
pub trait Interpolate {
    fn interpolate(x: f64, x_: &Vec<f64>, y_: &Vec<f64>, enable_extrapolation: bool) -> Result<f64>;
}
