use crate::{
    math::solver::traits::{CostFunction, SolverError},
    utils::errors::{AtlasError, Result},
};

// # BrentRoot result structure
/// Holds the result of the Brent's method root finding algorithm.
pub struct BrentRootResult {
    pub root: f64,
    pub iterations: usize,
    pub final_value: f64, // f(root)
    pub last_a: f64,      // valor de a al terminar
    pub last_b: f64,      // valor de b al terminar
}

pub struct BrentRoot<P>
where
    P: CostFunction,
{
    problem: P,
    a: f64,
    b: f64,
    tol: f64,
    max_iters: usize,
}

impl<P> BrentRoot<P>
where
    P: CostFunction,
{
    pub fn new(problem: P, a: f64, b: f64) -> Self {
        Self {
            problem,
            a,
            b,
            tol: 1e-7,
            max_iters: 500,
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

    pub fn solve(&self) -> Result<BrentRootResult> {
        if self.tol <= 0.0 {
            return Err(AtlasError::SolverError(
                SolverError::BrentRootError("Tolerance must be positive".to_string()),
            ));
        }

        // (Aquí va la lógica de Brent: igual que el ejemplo anterior)
        let mut a = self.a;
        let mut b = self.b;
        let mut fa = self.problem.cost(&a)?;
        let mut fb = self.problem.cost(&b)?;
        if fa * fb >= 0.0 {
            return Err(AtlasError::SolverError(
                SolverError::BrentRootError(format!(
                    "Function values at endpoints do not have opposite signs: f(a)={}, f(b)={}",
                    fa, fb
                )
            )))
        }

        let mut c = a;
        let mut fc = fa;
        let mut d = b - a;
        let mut e = d;

        for iter in 0..self.max_iters {
            if fb.abs() < self.tol {
                return Ok(
                    BrentRootResult {
                        root: b,
                        iterations: iter + 1,
                        final_value: fb,
                        last_a: a,
                        last_b: b,
                    }
                );

            }
            if fa.abs() < self.tol {
                return Ok(
                    BrentRootResult {
                        root: a,
                        iterations: iter + 1,
                        final_value: fa,
                        last_a: a,
                        last_b: b,
                    }
                );
            }
            if fa * fb < 0.0 {
                c = a;
                fc = fa;
                d = b - a;
                e = d;
            }
            if fc.abs() < fb.abs() {
                a = b;
                b = c;
                c = a;
                fa = fb;
                fb = fc;
                fc = fa;
            }
            let tol_act = 2.0 * std::f64::EPSILON * b.abs() + self.tol / 2.0;
            let m = 0.5 * (c - b);
            if fb.abs() < self.tol || m.abs() < tol_act {
                return Ok(
                    BrentRootResult {
                        root: b,
                        iterations: iter + 1,
                        final_value: fb,
                        last_a: a,
                        last_b: b,
                    }
                );
            }
            let mut p;
            let mut q;
            if (e.abs() >= tol_act) && (fa.abs() > fb.abs()) {
                let s = fb / fa;
                if a == c {
                    p = 2.0 * m * s;
                    q = 1.0 - s;
                } else {
                    let q1 = fa / fc;
                    let q2 = fb / fc;
                    p = s * (2.0 * m * q1 * (q1 - q2) - (b - a) * (q2 - 1.0));
                    q = (q1 - 1.0) * (q2 - 1.0) * (s - 1.0);
                }
                if p > 0.0 {
                    q = -q;
                }
                p = p.abs();
                if (2.0 * p) < (3.0 * m * q - (tol_act * q).abs()) && (p < (e.abs() * q / 2.0)) {
                    e = d;
                    d = p / q;
                } else {
                    d = m;
                    e = d;
                }
            } else {
                d = m;
                e = d;
            }
            a = b;
            fa = fb;
            if d.abs() > tol_act {
                b += d;
            } else {
                b += if m > 0.0 { tol_act } else { -tol_act };
            }
            fb = self.problem.cost(&b)?;
        }
        Err(AtlasError::SolverError(
            SolverError::MaxIterationsReached(
                "Maximum iterations reached without convergence".to_string(),
            ),
        ))
    }
}



#[cfg(test)]
mod tests {

    use super::*;

    struct Quadratic {
        // f(x) = x^2 - 2, root at sqrt(2)
    }
    impl CostFunction for Quadratic {
        fn cost(&self, x: &f64) -> Result<f64> {
            Ok(x * x - 2.0)
        }
    }

    struct Linear {
        // f(x) = x - 3, root at 3
    }
    impl CostFunction for Linear {
        fn cost(&self, x: &f64) -> Result<f64> {
            Ok(x - 3.0)
        }
    }

    struct NoRoot;
    impl CostFunction for NoRoot {
        fn cost(&self, x: &f64) -> Result<f64> {
            Ok(x * x + 1.0)
        }
    }

    #[test]
    fn test_brent_root_quadratic() {
        let problem = Quadratic {};
        let solver = BrentRoot::new(problem, 0.0, 2.0);
        let result = solver.solve().unwrap();
        assert!((result.root - 2.0_f64.sqrt()).abs() < 1e-8);
        assert!(result.final_value.abs() < 1e-8);
    }

    #[test]
    fn test_brent_root_linear() {
        let problem = Linear {};
        let solver = BrentRoot::new(problem, 0.0, 5.0);
        let result = solver.solve().unwrap();
        assert!((result.root - 3.0).abs() < 1e-8);
        assert!(result.final_value.abs() < 1e-8);
    }

    #[test]
    fn test_brent_root_no_root() {
        let problem = NoRoot {};
        let solver = BrentRoot::new(problem, -2.0, 2.0);
        let result = solver.solve();
        assert!(result.is_err());
    }

    #[test]
    fn test_brent_root_with_tolerance() {
        let problem = Quadratic {};
        let solver = BrentRoot::new(problem, 0.0, 2.0).with_tolerance(1e-4);
        let result = solver.solve().unwrap();
        assert!((result.root - 2.0_f64.sqrt()).abs() < 1e-4);
    }

    #[test]
    fn test_brent_root_max_iterations() {
        struct SlowFunc;
        impl CostFunction for SlowFunc {
            fn cost(&self, x: &f64) -> Result<f64> {
                Ok(x - 1e-4)
            }
        }
        let problem = SlowFunc {};
        let solver = BrentRoot::new(problem, 0.0, 2.0).with_max_iters(1);
        let result = solver.solve();
        assert!(result.is_err());
    }

    #[test]
    fn test_brent_root_negative_interval() {
        // Test with negative interval, root at -sqrt(2)
        struct NegQuadratic;
        impl CostFunction for NegQuadratic {
            fn cost(&self, x: &f64) -> Result<f64> {
                Ok(x * x - 2.0)
            }
        }
        let problem = NegQuadratic {};
        let solver = BrentRoot::new(problem, -2.0, 0.0);
        let result = solver.solve().unwrap();
        assert!((result.root + 2.0_f64.sqrt()).abs() < 1e-8);
        assert!(result.final_value.abs() < 1e-8);
    }

    #[test]
    fn test_brent_root_small_interval() {
        // Test with a very small interval containing the root
        struct SmallInterval;
        impl CostFunction for SmallInterval {
            fn cost(&self, x: &f64) -> Result<f64> {
                Ok(x - 0.5)
            }
        }
        let problem = SmallInterval {};
        let solver = BrentRoot::new(problem, 0.4, 0.6);
        let result = solver.solve().unwrap();
        assert!((result.root - 0.5).abs() < 1e-8);
        assert!(result.final_value.abs() < 1e-8);
    }

    #[test]
    fn test_brent_root_function_with_multiple_roots() {
        // f(x) = x^3 - x, roots at -1, 0, 1
        struct Cubic;
        impl CostFunction for Cubic {
            fn cost(&self, x: &f64) -> Result<f64> {
                Ok(x * x * x - x)
            }
        }
        let problem = Cubic {};
        // Interval [-2, -0.5] contains root at -1
        let solver = BrentRoot::new(problem, -2.0, -0.5);
        let result = solver.solve().unwrap();
        assert!((result.root + 1.0).abs() < 1e-8);

        // Interval [0.5, 2.0] contains root at 1
        let problem = Cubic {};
        let solver = BrentRoot::new(problem, 0.5, 2.0);
        let result = solver.solve().unwrap();
        assert!((result.root - 1.0).abs() < 1e-8);
    }

    #[test]
    fn test_brent_root_invalid_interval() {
        // Both endpoints have same sign, should error
        struct SameSign;
        impl CostFunction for SameSign {
            fn cost(&self, x: &f64) -> Result<f64> {
                Ok(x * x + 2.0)
            }
        }
        let problem = SameSign {};
        let solver = BrentRoot::new(problem, 1.0, 2.0);
        let result = solver.solve();
        assert!(result.is_err());
    }
}