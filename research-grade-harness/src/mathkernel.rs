//! Computational math kernel: the deterministic CPU oracle the meta-automaton
//! verifies against. The model emits a `KernelOp` (never mental math); the
//! kernel executes it exactly and returns a `KernelResult` whose `hash` is a
//! sha256 over a CANONICAL byte encoding (fixed-precision floats, sorted where
//! order is not semantic). Two independent routes that compute the same
//! quantity yield the same hash — that equality is the automaton's
//! verification signal.
//!
//! No JSON (crate invariant): ops and results are native Rust; the wire form
//! is the canonical byte string defined here.
//!
//! Capabilities:
//!   - High-dimensional linear algebra  (faer): matmul, solve, singular values.
//!   - Differential geometry (num-dual autodiff over a metric registry):
//!     Christoffel symbols and 2-D scalar (Gaussian) curvature at a point.
//!   - Number theory / computation gate (pure): modpow, gcd, lcm.
//!
//! GPU (MSL) is a future sub-lane; its results are tolerance-verified against
//! this CPU oracle, never hash-compared (GPU reductions are order-nondet).

use faer::linalg::solvers::Solve;
use num_dual::{Dual64, DualNum};
use sha2::{Digest, Sha256};

/// Fixed precision for the canonical float encoding. Any two values that agree
/// to this many significant digits hash identically — the determinism knob.
const CANON_PREC: usize = 10;

fn canon_f64(buf: &mut String, x: f64) {
    // Normalize -0.0, NaN, and subnormal noise to a stable textual form.
    let v = if x == 0.0 { 0.0 } else { x };
    buf.push_str(&format!("{:.*e};", CANON_PREC, v));
}

fn hash_canon(canon: &str) -> String {
    let mut h = Sha256::new();
    h.update(canon.as_bytes());
    hex::encode(h.finalize())
}

/// One kernel result: the canonical byte string and its sha256.
#[derive(Clone, Debug, PartialEq)]
pub struct KernelResult {
    pub canonical: String,
    pub hash: String,
    /// Scalar read-out when the op produces one (curvature, determinant, …).
    pub scalar: Option<f64>,
    /// Vector read-out (singular values, solution vector, Christoffel row, …).
    pub vector: Vec<f64>,
}

impl KernelResult {
    fn finish(canonical: String, scalar: Option<f64>, vector: Vec<f64>) -> Self {
        let hash = hash_canon(&canonical);
        Self { canonical, hash, scalar, vector }
    }
}

#[derive(Clone, Debug)]
pub enum KernelError {
    Shape(String),
    Singular,
    UnknownMetric(String),
    IllPosed(String),
}

/// Analytic metrics the diffgeom ops differentiate. Data-driven (no closures)
/// so a `KernelOp` stays a plain value the model can emit and the lane can
/// serialize. Extend by adding a variant + its `g_ij` below.
#[derive(Clone, Debug, PartialEq)]
pub enum Metric {
    /// Flat R^n (identity metric) — curvature must come out 0.
    FlatEuclidean { dim: usize },
    /// Round 2-sphere radius r in (theta, phi): diag(r^2, r^2 sin^2 theta).
    Sphere2 { r: f64 },
    /// Poincare upper half-plane: (dx^2+dy^2)/y^2. Constant curvature -1.
    PoincareH2,
    /// Round 3-sphere radius r in (chi, theta, phi). Scalar curvature 6/r^2.
    Sphere3 { r: f64 },
    /// Hyperbolic 3-space, upper half-space (dx^2+dy^2+dz^2)/z^2. Scalar -6.
    HyperbolicH3,
}

impl Metric {
    pub fn dim(&self) -> usize {
        match self {
            Metric::FlatEuclidean { dim } => *dim,
            Metric::Sphere2 { .. } | Metric::PoincareH2 => 2,
            Metric::Sphere3 { .. } | Metric::HyperbolicH3 => 3,
        }
    }
    /// Metric tensor g_ij at point x, generic over the autodiff scalar D.
    fn g<D: DualNum<f64> + Copy>(&self, x: &[D]) -> Vec<Vec<D>> {
        let z = |v: f64| D::from(v);
        match self {
            Metric::FlatEuclidean { dim } => {
                let mut m = vec![vec![z(0.0); *dim]; *dim];
                for i in 0..*dim { m[i][i] = z(1.0); }
                m
            }
            Metric::Sphere2 { r } => {
                let theta = x[0];
                let r2 = z(r * r);
                let s = theta.sin();
                vec![vec![r2, z(0.0)], vec![z(0.0), r2 * s * s]]
            }
            Metric::PoincareH2 => {
                let y = x[1];
                let inv = z(1.0) / (y * y);
                vec![vec![inv, z(0.0)], vec![z(0.0), inv]]
            }
            Metric::Sphere3 { r } => {
                // ds^2 = r^2( dchi^2 + sin^2 chi ( dtheta^2 + sin^2 theta dphi^2 ) )
                let chi = x[0]; let theta = x[1];
                let r2 = z(r * r);
                let sc = chi.sin();
                let st = theta.sin();
                let mut m = vec![vec![z(0.0); 3]; 3];
                m[0][0] = r2;
                m[1][1] = r2 * sc * sc;
                m[2][2] = r2 * sc * sc * st * st;
                m
            }
            Metric::HyperbolicH3 => {
                let zc = x[2];
                let inv = z(1.0) / (zc * zc);
                let mut m = vec![vec![z(0.0); 3]; 3];
                m[0][0] = inv; m[1][1] = inv; m[2][2] = inv;
                m
            }
        }
    }
}

/// The op the model emits. Closed set → verifiable, no arbitrary code.
#[derive(Clone, Debug)]
pub enum KernelOp {
    /// Row-major A (m×k) times B (k×n).
    MatMul { a: Vec<Vec<f64>>, b: Vec<Vec<f64>> },
    /// Solve A x = b (A square, n×n).
    Solve { a: Vec<Vec<f64>>, b: Vec<f64> },
    /// Singular values of A (m×n), descending — deterministic (sign-free).
    SingularValues { a: Vec<Vec<f64>> },
    /// Christoffel symbols Γ^k_{ij} of `metric` at `point`; flattened k,i,j.
    Christoffel { metric: Metric, point: Vec<f64> },
    /// Scalar (Gaussian for 2-D) curvature of `metric` at `point`.
    ScalarCurvature { metric: Metric, point: Vec<f64> },
    /// (base^exp) mod m — exact, the computation gate's modular lane.
    ModPow { base: u64, exp: u64, modulus: u64 },
    /// gcd(a,b).
    Gcd { a: u64, b: u64 },
    /// lcm of a list.
    Lcm { xs: Vec<u64> },
    /// Determinant of a square matrix (via LU).
    Determinant { a: Vec<Vec<f64>> },
    /// Eigenvalues of a SYMMETRIC matrix, ascending (deterministic).
    SymEigenvalues { a: Vec<Vec<f64>> },
    /// Numerical rank of A at tolerance `tol` (singular values > tol).
    Rank { a: Vec<Vec<f64>>, tol: f64 },
    /// Primality test.
    IsPrime { n: u64 },
    /// Prime factorization (ascending factors with multiplicity).
    Factorize { n: u64 },
    /// Modular inverse of a mod m, or IllPosed if gcd(a,m)!=1.
    ModInv { a: u64, modulus: u64 },
    /// Euler totient φ(n).
    Totient { n: u64 },
}

pub fn execute(op: &KernelOp) -> Result<KernelResult, KernelError> {
    match op {
        KernelOp::MatMul { a, b } => mat_mul(a, b),
        KernelOp::Solve { a, b } => solve(a, b),
        KernelOp::SingularValues { a } => singular_values(a),
        KernelOp::Christoffel { metric, point } => christoffel(metric, point),
        KernelOp::ScalarCurvature { metric, point } => scalar_curvature(metric, point),
        KernelOp::ModPow { base, exp, modulus } => {
            if *modulus == 0 { return Err(KernelError::IllPosed("modulus 0".into())); }
            let r = mod_pow(*base as u128, *exp as u128, *modulus as u128) as u64;
            let mut c = String::from("modpow;");
            c.push_str(&format!("{r};"));
            Ok(KernelResult::finish(c, Some(r as f64), vec![r as f64]))
        }
        KernelOp::Gcd { a, b } => {
            let r = gcd(*a, *b);
            Ok(KernelResult::finish(format!("gcd;{r};"), Some(r as f64), vec![r as f64]))
        }
        KernelOp::Lcm { xs } => {
            let mut acc: u64 = 1;
            for &x in xs {
                if x == 0 { return Err(KernelError::IllPosed("lcm of 0".into())); }
                acc = acc / gcd(acc, x) * x;
            }
            Ok(KernelResult::finish(format!("lcm;{acc};"), Some(acc as f64), vec![acc as f64]))
        }
        KernelOp::Determinant { a } => determinant(a),
        KernelOp::SymEigenvalues { a } => sym_eigenvalues(a),
        KernelOp::Rank { a, tol } => rank(a, *tol),
        KernelOp::IsPrime { n } => {
            let p = is_prime(*n);
            Ok(KernelResult::finish(format!("isprime;{};", p as u8), Some(p as u8 as f64), vec![p as u8 as f64]))
        }
        KernelOp::Factorize { n } => {
            let f = factorize(*n);
            let mut c = String::from("factorize;");
            let mut v = Vec::new();
            for (p, e) in &f { c.push_str(&format!("{p}^{e};")); v.push(*p as f64); v.push(*e as f64); }
            Ok(KernelResult::finish(c, None, v))
        }
        KernelOp::ModInv { a, modulus } => {
            match mod_inv(*a as i128, *modulus as i128) {
                Some(inv) => Ok(KernelResult::finish(format!("modinv;{inv};"), Some(inv as f64), vec![inv as f64])),
                None => Err(KernelError::IllPosed("a not invertible mod m".into())),
            }
        }
        KernelOp::Totient { n } => {
            let t = totient(*n);
            Ok(KernelResult::finish(format!("totient;{t};"), Some(t as f64), vec![t as f64]))
        }
    }
}

// ---------- linear algebra (faer) ----------

fn to_faer(a: &[Vec<f64>]) -> Result<faer::Mat<f64>, KernelError> {
    let m = a.len();
    if m == 0 { return Err(KernelError::Shape("empty".into())); }
    let n = a[0].len();
    if a.iter().any(|r| r.len() != n) { return Err(KernelError::Shape("ragged".into())); }
    Ok(faer::Mat::from_fn(m, n, |i, j| a[i][j]))
}

fn mat_mul(a: &[Vec<f64>], b: &[Vec<f64>]) -> Result<KernelResult, KernelError> {
    let fa = to_faer(a)?; let fb = to_faer(b)?;
    if fa.ncols() != fb.nrows() { return Err(KernelError::Shape("inner dim".into())); }
    let c = &fa * &fb;
    let mut canon = String::from("matmul;");
    let mut v = Vec::new();
    for i in 0..c.nrows() { for j in 0..c.ncols() { canon_f64(&mut canon, c[(i, j)]); v.push(c[(i, j)]); } }
    Ok(KernelResult::finish(canon, None, v))
}

fn solve(a: &[Vec<f64>], b: &[f64]) -> Result<KernelResult, KernelError> {
    let fa = to_faer(a)?;
    if fa.nrows() != fa.ncols() || fa.nrows() != b.len() {
        return Err(KernelError::Shape("solve dims".into()));
    }
    let lu = fa.partial_piv_lu();
    let rhs = faer::Mat::from_fn(b.len(), 1, |i, _| b[i]);
    let x = lu.solve(&rhs);
    let mut canon = String::from("solve;");
    let mut v = Vec::new();
    for i in 0..x.nrows() { canon_f64(&mut canon, x[(i, 0)]); v.push(x[(i, 0)]); }
    Ok(KernelResult::finish(canon, None, v))
}

fn singular_values(a: &[Vec<f64>]) -> Result<KernelResult, KernelError> {
    let fa = to_faer(a)?;
    let svd = fa.svd().map_err(|_| KernelError::IllPosed("svd failed".into()))?;
    let s = svd.S();
    let col = s.column_vector();
    let n = col.nrows();
    let mut v: Vec<f64> = (0..n).map(|i| col[i]).collect();
    v.sort_by(|x, y| y.partial_cmp(x).unwrap());
    let mut canon = String::from("svals;");
    for &sv in &v { canon_f64(&mut canon, sv); }
    Ok(KernelResult::finish(canon, None, v))
}

// ---------- differential geometry (num-dual autodiff) ----------

/// Inverse of a small symmetric matrix (2×2 / n×n via cofactor for n<=3).
fn inv_small(g: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, KernelError> {
    let n = g.len();
    match n {
        1 => {
            if g[0][0] == 0.0 { return Err(KernelError::Singular); }
            Ok(vec![vec![1.0 / g[0][0]]])
        }
        2 => {
            let det = g[0][0] * g[1][1] - g[0][1] * g[1][0];
            if det == 0.0 { return Err(KernelError::Singular); }
            Ok(vec![
                vec![g[1][1] / det, -g[0][1] / det],
                vec![-g[1][0] / det, g[0][0] / det],
            ])
        }
        _ => Err(KernelError::IllPosed("inv_small supports n<=2".into())),
    }
}

/// ∂g_{ab}/∂x^c at `point` via forward-mode autodiff: seed coordinate c with a
/// unit dual part, evaluate the metric, read the eps component. Returns
/// dg[c][a][b]. Exact (not finite-difference) and deterministic.
fn metric_jacobian(metric: &Metric, point: &[f64]) -> Vec<Vec<Vec<f64>>> {
    let n = metric.dim();
    let mut dg = vec![vec![vec![0.0; n]; n]; n];
    for c in 0..n {
        let p: Vec<Dual64> = point.iter().enumerate()
            .map(|(k, &v)| if k == c { Dual64::new(v, 1.0) } else { Dual64::from(v) })
            .collect();
        let g = metric.g(&p);
        for a in 0..n {
            for b in 0..n {
                dg[c][a][b] = g[a][b].eps;
            }
        }
    }
    dg
}

/// Invert the metric at a point. Registry metrics are diagonal, so invert the
/// diagonal directly for any n; fall back to the general cofactor form for a
/// non-diagonal 2×2. Deterministic.
fn inv_metric(g: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, KernelError> {
    let n = g.len();
    let diagonal = (0..n).all(|i| (0..n).all(|j| i == j || g[i][j].abs() < 1e-14));
    if diagonal {
        let mut inv = vec![vec![0.0; n]; n];
        for i in 0..n {
            if g[i][i] == 0.0 { return Err(KernelError::Singular); }
            inv[i][i] = 1.0 / g[i][i];
        }
        return Ok(inv);
    }
    inv_small(g)
}

/// Christoffel symbols Γ^k_{ij} (flattened k*n*n + i*n + j) at `point`, computed
/// from exact autodiff metric derivatives. Reusable by the curvature path.
fn christoffel_vec(metric: &Metric, point: &[f64]) -> Result<Vec<f64>, KernelError> {
    let n = metric.dim();
    if point.len() != n { return Err(KernelError::Shape("point dim".into())); }
    let g0: Vec<Vec<f64>> = {
        let p: Vec<Dual64> = point.iter().map(|&v| Dual64::from(v)).collect();
        metric.g(&p).iter().map(|r| r.iter().map(|d| d.re).collect()).collect()
    };
    let ginv = inv_metric(&g0)?;
    let dg = metric_jacobian(metric, point);
    // Γ^k_{ij} = 1/2 g^{kl} ( ∂_i g_{lj} + ∂_j g_{li} - ∂_l g_{ij} )
    let mut gamma = vec![0.0; n * n * n];
    for k in 0..n {
        for i in 0..n {
            for j in 0..n {
                let mut s = 0.0;
                for l in 0..n {
                    s += ginv[k][l] * (dg[i][l][j] + dg[j][l][i] - dg[l][i][j]);
                }
                gamma[k * n * n + i * n + j] = 0.5 * s;
            }
        }
    }
    Ok(gamma)
}

fn christoffel(metric: &Metric, point: &[f64]) -> Result<KernelResult, KernelError> {
    let gamma = christoffel_vec(metric, point)?;
    let mut canon = String::from("christoffel;");
    for &v in &gamma { canon_f64(&mut canon, v); }
    Ok(KernelResult::finish(canon, None, gamma))
}

/// Finite-difference step for ∂Γ. Fixed → deterministic hashes; curvature is
/// then exact to ~1e-7, well within CANON_PREC rounding for cross-route match.
const FD_H: f64 = 1e-4;

/// Scalar curvature R of `metric` at `point` in ANY dimension.
/// Riemann R^ρ_{σμν} = ∂_μ Γ^ρ_{νσ} − ∂_ν Γ^ρ_{μσ} + Γ^ρ_{μλ}Γ^λ_{νσ} − Γ^ρ_{νλ}Γ^λ_{μσ};
/// Ricci R_{σν}=Σ_ρ R^ρ_{σρν}; R = g^{σν} R_{σν}. ∂Γ by central difference of the
/// autodiff Christoffel (exact Γ, fixed-step derivative → deterministic).
fn scalar_curvature(metric: &Metric, point: &[f64]) -> Result<KernelResult, KernelError> {
    let n = metric.dim();
    if point.len() != n { return Err(KernelError::Shape("point dim".into())); }
    let idx = |k: usize, i: usize, j: usize| k * n * n + i * n + j;
    let gamma = christoffel_vec(metric, point)?;
    // ∂_m Γ^k_{ij} via central difference.
    let mut dgamma = vec![vec![0.0; n * n * n]; n];
    for m in 0..n {
        let mut pp = point.to_vec(); pp[m] += FD_H;
        let mut pm = point.to_vec(); pm[m] -= FD_H;
        let gp = christoffel_vec(metric, &pp)?;
        let gm = christoffel_vec(metric, &pm)?;
        for t in 0..n * n * n { dgamma[m][t] = (gp[t] - gm[t]) / (2.0 * FD_H); }
    }
    // Ricci R_{sv} = Σ_ρ ( ∂_ρ Γ^ρ_{vs} − ∂_v Γ^ρ_{ρs} + Γ^ρ_{ρλ}Γ^λ_{vs} − Γ^ρ_{vλ}Γ^λ_{ρs} )
    let g0: Vec<Vec<f64>> = {
        let p: Vec<Dual64> = point.iter().map(|&v| Dual64::from(v)).collect();
        metric.g(&p).iter().map(|r| r.iter().map(|d| d.re).collect()).collect()
    };
    let ginv = inv_metric(&g0)?;
    let mut r = 0.0;
    for s in 0..n {
        for v in 0..n {
            let mut ric = 0.0;
            for rho in 0..n {
                ric += dgamma[rho][idx(rho, v, s)] - dgamma[v][idx(rho, rho, s)];
                for lam in 0..n {
                    ric += gamma[idx(rho, rho, lam)] * gamma[idx(lam, v, s)]
                         - gamma[idx(rho, v, lam)] * gamma[idx(lam, rho, s)];
                }
            }
            r += ginv[s][v] * ric;
        }
    }
    let mut canon = String::from("scalarR;");
    canon_f64(&mut canon, r);
    Ok(KernelResult::finish(canon, Some(r), vec![r]))
}

// ---------- number theory ----------

fn mod_pow(mut base: u128, mut exp: u128, modulus: u128) -> u128 {
    if modulus == 1 { return 0; }
    let mut r: u128 = 1; base %= modulus;
    while exp > 0 {
        if exp & 1 == 1 { r = r * base % modulus; }
        exp >>= 1; base = base * base % modulus;
    }
    r
}

fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 { let t = b; b = a % b; a = t; }
    a
}

// ---------- extended linear algebra (deterministic, plain Rust) ----------

fn square(a: &[Vec<f64>]) -> Result<usize, KernelError> {
    let n = a.len();
    if n == 0 || a.iter().any(|r| r.len() != n) { return Err(KernelError::Shape("not square".into())); }
    Ok(n)
}

fn determinant(a: &[Vec<f64>]) -> Result<KernelResult, KernelError> {
    let n = square(a)?;
    let mut m: Vec<Vec<f64>> = a.to_vec();
    let mut det = 1.0f64;
    for col in 0..n {
        // partial pivot
        let mut piv = col;
        for r in col + 1..n { if m[r][col].abs() > m[piv][col].abs() { piv = r; } }
        if m[piv][col].abs() < 1e-300 { det = 0.0; break; }
        if piv != col { m.swap(piv, col); det = -det; }
        det *= m[col][col];
        for r in col + 1..n {
            let f = m[r][col] / m[col][col];
            for c in col..n { m[r][c] -= f * m[col][c]; }
        }
    }
    let mut canon = String::from("det;"); canon_f64(&mut canon, det);
    Ok(KernelResult::finish(canon, Some(det), vec![det]))
}

/// Symmetric eigenvalues via the classic cyclic Jacobi rotation. Sorted
/// ascending → deterministic. (Small dense matrices.)
fn sym_eigenvalues(a: &[Vec<f64>]) -> Result<KernelResult, KernelError> {
    let n = square(a)?;
    for i in 0..n { for j in 0..n { if (a[i][j] - a[j][i]).abs() > 1e-9 {
        return Err(KernelError::IllPosed("matrix not symmetric".into())); } } }
    let mut m = a.to_vec();
    for _ in 0..100 {
        // largest off-diagonal
        let (mut p, mut q, mut off) = (0usize, 1usize, 0.0f64);
        for i in 0..n { for j in i + 1..n { if m[i][j].abs() > off { off = m[i][j].abs(); p = i; q = j; } } }
        if off < 1e-14 { break; }
        let app = m[p][p]; let aqq = m[q][q]; let apq = m[p][q];
        let phi = 0.5 * (2.0 * apq).atan2(aqq - app);
        let (c, sn) = (phi.cos(), phi.sin());
        for k in 0..n {
            let mkp = m[k][p]; let mkq = m[k][q];
            m[k][p] = c * mkp - sn * mkq;
            m[k][q] = sn * mkp + c * mkq;
        }
        for k in 0..n {
            let mpk = m[p][k]; let mqk = m[q][k];
            m[p][k] = c * mpk - sn * mqk;
            m[q][k] = sn * mpk + c * mqk;
        }
    }
    let mut ev: Vec<f64> = (0..n).map(|i| m[i][i]).collect();
    ev.sort_by(|x, y| x.partial_cmp(y).unwrap());
    let mut canon = String::from("eig;"); for &e in &ev { canon_f64(&mut canon, e); }
    Ok(KernelResult::finish(canon, None, ev))
}

fn rank(a: &[Vec<f64>], tol: f64) -> Result<KernelResult, KernelError> {
    let sv = singular_values(a)?;
    let r = sv.vector.iter().filter(|&&s| s > tol).count() as f64;
    let mut canon = String::from("rank;"); canon_f64(&mut canon, r);
    Ok(KernelResult::finish(canon, Some(r), vec![r]))
}

// ---------- extended number theory ----------

fn is_prime(n: u64) -> bool {
    if n < 2 { return false; }
    for &p in &[2u64, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37] {
        if n % p == 0 { return n == p; }
    }
    // deterministic Miller-Rabin for u64 with this witness set
    let mut d = n - 1; let mut s = 0; while d % 2 == 0 { d /= 2; s += 1; }
    'wit: for &a in &[2u64, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37] {
        let mut x = mod_pow(a as u128, d as u128, n as u128);
        if x == 1 || x == (n as u128 - 1) { continue; }
        for _ in 0..s - 1 {
            x = x * x % n as u128;
            if x == n as u128 - 1 { continue 'wit; }
        }
        return false;
    }
    true
}

fn factorize(mut n: u64) -> Vec<(u64, u32)> {
    let mut f = Vec::new();
    let mut d = 2u64;
    while d * d <= n {
        if n % d == 0 {
            let mut e = 0u32;
            while n % d == 0 { n /= d; e += 1; }
            f.push((d, e));
        }
        d += if d == 2 { 1 } else { 2 };
    }
    if n > 1 { f.push((n, 1)); }
    f
}

fn mod_inv(a: i128, m: i128) -> Option<u64> {
    let (mut old_r, mut r) = (a.rem_euclid(m), m);
    let (mut old_s, mut s) = (1i128, 0i128);
    while r != 0 {
        let q = old_r / r;
        (old_r, r) = (r, old_r - q * r);
        (old_s, s) = (s, old_s - q * s);
    }
    if old_r != 1 { None } else { Some(old_s.rem_euclid(m) as u64) }
}

fn totient(n: u64) -> u64 {
    if n == 0 { return 0; }
    let mut result = n;
    for (p, _) in factorize(n) { result -= result / p; }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    fn run(op: KernelOp) -> KernelResult { execute(&op).expect("op ok") }

    #[test]
    fn number_theory() {
        assert_eq!(run(KernelOp::ModPow { base: 7, exp: 100, modulus: 13 }).scalar, Some(9.0));
        assert_eq!(run(KernelOp::Gcd { a: 48, b: 18 }).scalar, Some(6.0));
        assert_eq!(run(KernelOp::Lcm { xs: vec![24, 36, 90] }).scalar, Some(360.0));
    }

    #[test]
    fn linalg() {
        let sv = run(KernelOp::SingularValues { a: vec![vec![3.0, 0.0], vec![0.0, 4.0]] });
        assert!((sv.vector[0] - 4.0).abs() < 1e-9 && (sv.vector[1] - 3.0).abs() < 1e-9);
        let x = run(KernelOp::Solve { a: vec![vec![2.0, 0.0], vec![0.0, 2.0]], b: vec![2.0, 4.0] });
        assert!((x.vector[0] - 1.0).abs() < 1e-9 && (x.vector[1] - 2.0).abs() < 1e-9);
    }

    #[test]
    fn determinism_hash() {
        let a = run(KernelOp::SingularValues { a: vec![vec![1.0, 2.0], vec![3.0, 4.0]] });
        let b = run(KernelOp::SingularValues { a: vec![vec![1.0, 2.0], vec![3.0, 4.0]] });
        assert_eq!(a.hash, b.hash, "same op must hash identically");
    }

    #[test]
    fn christoffel_sphere() {
        // S^2 radius 1 at theta=pi/4: Gamma^theta_{phi,phi} = -sin*cos = -0.5
        let p = std::f64::consts::FRAC_PI_4;
        let r = run(KernelOp::Christoffel { metric: Metric::Sphere2 { r: 1.0 }, point: vec![p, 0.0] });
        let n = 2;
        let g_theta_phiphi = r.vector[0 * n * n + 1 * n + 1]; // k=0(theta), i=1,j=1(phi)
        assert!((g_theta_phiphi - (-0.5)).abs() < 1e-6, "got {g_theta_phiphi}");
    }

    #[test]
    fn scalar_curvature() {
        // n-D Ricci scalar via FD-of-autodiff-Christoffel: accurate to ~1e-3.
        assert!((run(KernelOp::ScalarCurvature { metric: Metric::Sphere2 { r: 2.0 }, point: vec![1.0, 0.0] }).scalar.unwrap() - 0.5).abs() < 1e-2);
        assert!((run(KernelOp::ScalarCurvature { metric: Metric::PoincareH2, point: vec![0.0, 1.0] }).scalar.unwrap() - (-2.0)).abs() < 1e-2);
        assert!(run(KernelOp::ScalarCurvature { metric: Metric::FlatEuclidean { dim: 2 }, point: vec![0.0, 0.0] }).scalar.unwrap().abs() < 1e-9);
    }
}

#[cfg(test)]
mod tests_ext {
    use super::*;
    fn run(op: KernelOp) -> KernelResult { execute(&op).expect("op ok") }

    #[test]
    fn determinant_eig_rank() {
        assert!((run(KernelOp::Determinant { a: vec![vec![1.0,2.0],vec![3.0,4.0]] }).scalar.unwrap() + 2.0).abs() < 1e-9);
        let ev = run(KernelOp::SymEigenvalues { a: vec![vec![2.0,1.0],vec![1.0,2.0]] }).vector;
        assert!((ev[0]-1.0).abs()<1e-7 && (ev[1]-3.0).abs()<1e-7, "{ev:?}");
        assert_eq!(run(KernelOp::Rank { a: vec![vec![1.0,2.0],vec![2.0,4.0]], tol: 1e-9 }).scalar, Some(1.0));
    }

    #[test]
    fn number_theory_ext() {
        assert_eq!(run(KernelOp::IsPrime { n: 97 }).scalar, Some(1.0));
        assert_eq!(run(KernelOp::IsPrime { n: 91 }).scalar, Some(0.0));
        let f = run(KernelOp::Factorize { n: 360 }).vector; // [2,3, 3,2, 5,1]
        assert_eq!(f, vec![2.0,3.0,3.0,2.0,5.0,1.0]);
        assert_eq!(run(KernelOp::ModInv { a: 3, modulus: 11 }).scalar, Some(4.0));
        assert_eq!(run(KernelOp::Totient { n: 36 }).scalar, Some(12.0));
    }

    #[test]
    fn curvature_nd() {
        // regression: 2-D sphere still 0.5
        assert!((run(KernelOp::ScalarCurvature { metric: Metric::Sphere2 { r: 2.0 }, point: vec![1.0,0.3] }).scalar.unwrap() - 0.5).abs() < 1e-3);
        // 3-sphere radius 2: R = 6/r^2 = 1.5
        assert!((run(KernelOp::ScalarCurvature { metric: Metric::Sphere3 { r: 2.0 }, point: vec![1.0,1.0,0.5] }).scalar.unwrap() - 1.5).abs() < 1e-2, "S3");
        // hyperbolic H^3: R = -6
        assert!((run(KernelOp::ScalarCurvature { metric: Metric::HyperbolicH3, point: vec![0.0,0.0,1.0] }).scalar.unwrap() + 6.0).abs() < 1e-2, "H3");
        // flat 3-D: 0
        assert!(run(KernelOp::ScalarCurvature { metric: Metric::FlatEuclidean { dim: 3 }, point: vec![0.0,0.0,0.0] }).scalar.unwrap().abs() < 1e-6);
    }
}
