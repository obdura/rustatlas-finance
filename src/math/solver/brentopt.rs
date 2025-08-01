use crate::math::solver::traits::{CostFunction, SolverError};
use crate::utils::errors::{AtlasError, Result};

#[derive(Debug)]
pub struct BrentOptResult {
    pub argmin: f64,
    pub minimum: f64,
    pub iterations: usize,
    pub last_a: f64,
    pub last_b: f64,
}

pub struct BrentOpt<P>
where
    P: CostFunction,
{
    problem: P,
    a: f64,
    b: f64,
    tol: f64,
    max_iters: usize,
}

impl<P: CostFunction> BrentOpt<P> {
    pub fn new(problem: P, a: f64, b: f64) -> Self {
        Self {
            problem,
            a,
            b,
            tol: 1e-8,
            max_iters: 100,
        }
    }

    pub fn with_tolerance(mut self, tol: f64) -> Self {
        self.tol = tol;
        self
    }

    pub fn with_max_iters(mut self, max_iters: usize) -> Self {
        self.max_iters = max_iters;
        self
    }

    pub fn solve(&self) -> Result<BrentOptResult> {
        use std::f64::EPSILON;

        let (mut a, mut c) = (self.a, self.b);
        let mut x: f64 = a + (c - a) * 0.3819660112501051; // Golden ratio
        let mut w: f64 = x;
        let mut v: f64 = x;
        let mut fx: f64 = self.problem.cost(&x)?;
        let mut fw: f64 = fx;
        let mut fv: f64 = fx;
        let mut d: f64;
        let mut e: f64 = 0.0;

        let tol = self.tol;
        let mut iter;

        for i in 0..self.max_iters {
            iter = i + 1;
            let m = 0.5 * (a + c);
            let tol1 = tol * x.abs() + EPSILON;
            let tol2 = 2.0 * tol1;

            // Check for convergence
            if (x - m).abs() <= tol2 - 0.5 * (c - a) {
                return Ok(BrentOptResult {
                    argmin: x,
                    minimum: fx,
                    iterations: iter,
                    last_a: a,
                    last_b: c,
                });
            }

            let mut p;
            let mut q;
            let r;
            let mut u;

            if e.abs() > tol1 {
                // Fit parabola
                r = (x - w) * (fx - fv);
                q = (x - v) * (fx - fw);
                p = (x - v) * q - (x - w) * r;
                q = 2.0 * (q - r);

                if q > 0.0 {
                    p = -p;
                }
                q = q.abs();

                if (p.abs() < (0.5 * q * e)) && (p > q * (a - x)) && (p < q * (c - x)) {
                    // Parabolic step
                    d = p / q;
                    u = x + d;

                    // u must not be too close to a or c
                    if (u - a) < tol2 || (c - u) < tol2 {
                        d = if m - x >= 0.0 { tol1 } else { -tol1 };
                    }
                } else {
                    // Golden section
                    e = if x >= m { a - x } else { c - x };
                    d = 0.618 * e;
                }
            } else {
                // Golden section
                e = if x >= m { a - x } else { c - x };
                d = 0.618 * e;
            }

            u = if d.abs() >= tol1 {
                x + d
            } else {
                x + tol1.copysign(d)
            };
            let fu = self.problem.cost(&u)?;

            // Update a, c
            if fu <= fx {
                if u < x {
                    c = x;
                } else {
                    a = x;
                }
                v = w;
                fv = fw;
                w = x;
                fw = fx;
                x = u;
                fx = fu;
            } else {
                if u < x {
                    a = u;
                } else {
                    c = u;
                }
                if fu <= fw || w == x {
                    v = w;
                    fv = fw;
                    w = u;
                    fw = fu;
                } else if fu <= fv || v == x || v == w {
                    v = u;
                    fv = fu;
                }
            }
        }

        Err(AtlasError::SolverError(SolverError::MaxIterationsReached(
            format!(
                "BrentOpt did not converge after {} iterations",
                self.max_iters
            ),
        )))
    }
}



#[cfg(test)]
mod tests {
use super::*;

struct Quadratic;
impl CostFunction for Quadratic {
    fn cost(&self, x: &f64) -> Result<f64> {
        Ok((x - 2.0).powi(2) + 1.0)
    }
}

struct Cubic;
impl CostFunction for Cubic {
    fn cost(&self, x: &f64) -> Result<f64> {
        Ok((x + 1.0).powi(3) + 2.0)
    }
}

struct Flat;
impl CostFunction for Flat {
    fn cost(&self, _: &f64) -> Result<f64> {
        Ok(42.0)
    }
}

#[test]
fn test_brentopt_quadratic() {
    let solver = BrentOpt::new(Quadratic, 0.0, 5.0);
    let result = solver.solve().unwrap();
    assert!((result.argmin - 2.0).abs() < 1e-6);
    assert!((result.minimum - 1.0).abs() < 1e-6);
    assert!(result.iterations > 0);
}

#[test]
fn test_brentopt_cubic() {
    let solver = BrentOpt::new(Cubic, -3.0, 3.0);
    let result = solver.solve().unwrap();
    // Minimum is at the left endpoint for this cubic in [-3,3]
    assert!((result.argmin + 3.0).abs() < 1e-6);
    assert!(result.iterations > 0);
}

#[test]
fn test_brentopt_flat_function() {
    let solver = BrentOpt::new(Flat, -10.0, 10.0);
    let result = solver.solve().unwrap();
    assert!((result.minimum - 42.0).abs() < 1e-8);
    assert!(result.iterations > 0);
}

#[test]
fn test_brentopt_max_iterations() {
    struct Slow;
    impl CostFunction for Slow {
        fn cost(&self, _: &f64) -> Result<f64> {
            Ok(0.0)
        }
    }
    let solver = BrentOpt::new(Slow, 0.0, 1.0).with_max_iters(1);
    let err = solver.solve().unwrap_err();
    match err {
        AtlasError::SolverError(SolverError::MaxIterationsReached(_)) => {}
        _ => panic!("Expected MaxIterationsReached error"),
    }
}

#[test]
fn test_brentopt_with_tolerance() {
    let solver = BrentOpt::new(Quadratic, 0.0, 5.0).with_tolerance(1e-4);
    let result = solver.solve().unwrap();
    assert!((result.argmin - 2.0).abs() < 1e-3);
}
}   