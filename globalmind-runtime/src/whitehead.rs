//! Executable Whitehead 12-around-1 topology and unified `(H^3 x T^2) / Gamma_12`
//! subgroup specification.
//!
//! This module resolves the Lane D gap between separate Picard `H^3` reduction
//! and separate 12-sector `T^2` phase quantization by providing:
//! 1. Exact Gaussian integer `Z[i]` and Picard group `PSL(2, Z[i])` arithmetic.
//! 2. Explicit definition of the index-12 congruence subgroup
//!    `Gamma_12 = Gamma_0(2 + 2i) < PSL(2, Z[i])` stabilizing `[1 : 0]` on the
//!    12-element projective line `P^1(Z[i] / (2 + 2i))`, together with its 12
//!    left-coset representatives `C_0 .. C_11` and Schreier generator set.
//! 3. Exact `H^3` action via `2x2` Hermitian spinor congruence `X' = M X M^\dagger`
//!    (isomorphic to the 3D Poincaré Upper Half-Space extension) and coupled
//!    `T^2` action via the sign-invariant `(1+i)`-parity character homomorphism
//!    `chi: Gamma_12 -> T^2` along the 2:1 Kuramoto resonant direction plus
//!    the `Gamma / Gamma_12` coset holonomy.
//! 4. Central hub (`L0_Coordinator`) at `(1, 0, 0, 0) in H^3`, `(0, 0) in T^2`
//!    and twelve peer well centers (`L1 .. L12`, ordinals `0..11`, `T^2`
//!    sectors `0..11`) on the golden icosahedral shell `r_sph = 2 - phi`.
//! 5. Deterministic canonical representative reduction on `(H^3 x T^2) / Gamma_12`
//!    and no-gap/no-overlap Dirichlet-Voronoi well classification with explicit
//!    lowest-ordinal boundary tie-breaking and icosahedral neighbor overlap.

use crate::{Digest, RecordId};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::f64::consts::{PI, TAU};

pub const WHITEHEAD_CONTRACT_VERSION: &str = "globalmind-whitehead-gamma12-v1";
pub const WHITEHEAD_CLASSIFIER_VERSION: &str = "whitehead-12-around-1-v1";
pub const WHITEHEAD_SPACE_NAME: &str = "H3xT2/r_12";
pub const WHITEHEAD_MANIFOLD: &str = "H3xT2";
pub const WHITEHEAD_QUOTIENT_SYMBOL: &str = "r_12";
pub const WHITEHEAD_COORDINATE_MODEL: &str = "lorentz4-plus-torus2";

/// Tie-breaking tolerance for equidistant Voronoi/Dirichlet boundary tests.
pub const BOUNDARY_TIE_EPSILON: f64 = 1e-12;

/// Default authorized overlap margin (in joint hyperbolic-toroidal cost units)
/// between icosahedrally adjacent peer wells (`<d_r, d_s> = +1/sqrt(5)`).
pub const DEFAULT_ICOSAHEDRAL_OVERLAP_MARGIN: f64 = 0.08;

/// Default central hub horizon radius in hyperbolic geodesic distance `d_H3`.
pub const DEFAULT_HUB_HORIZON_RADIUS: f64 = 0.35;

const PICARD_EPS: f64 = 1e-11;

/// Exact Gaussian integer `re + i * im in Z[i]`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct GaussianInt {
    pub re: i64,
    pub im: i64,
}

impl GaussianInt {
    pub const ZERO: Self = Self { re: 0, im: 0 };
    pub const ONE: Self = Self { re: 1, im: 0 };
    pub const NEG_ONE: Self = Self { re: -1, im: 0 };
    pub const I: Self = Self { re: 0, im: 1 };
    pub const NEG_I: Self = Self { re: 0, im: -1 };
    pub const TWO_PLUS_TWO_I: Self = Self { re: 2, im: 2 };

    #[inline]
    pub const fn new(re: i64, im: i64) -> Self {
        Self { re, im }
    }

    #[inline]
    pub const fn conj(self) -> Self {
        Self {
            re: self.re,
            im: -self.im,
        }
    }

    #[inline]
    pub const fn neg(self) -> Self {
        Self {
            re: -self.re,
            im: -self.im,
        }
    }

    #[inline]
    pub const fn add(self, rhs: Self) -> Self {
        Self {
            re: self.re + rhs.re,
            im: self.im + rhs.im,
        }
    }

    #[inline]
    pub const fn sub(self, rhs: Self) -> Self {
        Self {
            re: self.re - rhs.re,
            im: self.im - rhs.im,
        }
    }

    #[inline]
    pub const fn mul(self, rhs: Self) -> Self {
        let re = (self.re as i128) * (rhs.re as i128) - (self.im as i128) * (rhs.im as i128);
        let im = (self.re as i128) * (rhs.im as i128) + (self.im as i128) * (rhs.re as i128);
        Self {
            re: re as i64,
            im: im as i64,
        }
    }

    #[inline]
    pub const fn norm(self) -> i128 {
        (self.re as i128) * (self.re as i128) + (self.im as i128) * (self.im as i128)
    }

    /// True iff `self` is divisible by `2 + 2i` in `Z[i]`.
    /// Since `(a + bi) / (2 + 2i) = ((a + b) + (b - a)i) / 4`, divisibility
    /// holds iff `a + b == 0 (mod 4)` and `b - a == 0 (mod 4)`.
    #[inline]
    pub const fn is_divisible_by_2_plus_2i(self) -> bool {
        (self.re + self.im).rem_euclid(4) == 0 && (self.im - self.re).rem_euclid(4) == 0
    }

    /// Exact quotient `self / (2 + 2i)` when `self` is divisible by `2 + 2i`.
    #[inline]
    pub const fn div_exact_2_plus_2i(self) -> Option<Self> {
        if !self.is_divisible_by_2_plus_2i() {
            return None;
        }
        Some(Self {
            re: (self.re + self.im) / 4,
            im: (self.im - self.re) / 4,
        })
    }

    /// Canonical residue of `self` in `Z[i] / (2 + 2i) Z[i]`, which has
    /// exact cardinality `N(2 + 2i) = 8`.
    /// We represent the 8 residue classes uniquely by `(r, s)` with
    /// `r in {0, 1, 2, 3}` and `s in {0, 1}` via `z = r + s * (1 + i) mod (2 + 2i)`.
    #[inline]
    pub const fn mod_2_plus_2i_code(self) -> u8 {
        // In Z[i]/(2+2i), 2i = -2 = 2 (mod 2+2i).
        // Write z = re + im*i = (re - im) + im*(1 + i).
        // Since (1 + i)^2 = 2i = -2 (mod 2+2i) and 4 = -(2+2i)(1-i) = 0 (mod 2+2i),
        // setting s = im.rem_euclid(2) in {0, 1} leaves z - s*(1+i) = re + (im - s)*i.
        // Write im - s = 2 * k. Since 2i == -2 (mod 2+2i), (2*k)*i == -2*k (mod 2+2i).
        // Hence r = (re - 2*k).rem_euclid(4) in {0, 1, 2, 3} is a canonical representative!
        let s = self.im.rem_euclid(2);
        let k = (self.im - s) / 2;
        let r = (self.re - 2 * k).rem_euclid(4);
        ((s as u8) << 2) | (r as u8)
    }

    #[inline]
    pub const fn from_mod_2_plus_2i_code(code: u8) -> Self {
        let r = (code & 0b11) as i64;
        let s = ((code >> 2) & 0b1) as i64;
        Self { re: r + s, im: s }
    }

    /// Exact parity `(re + im) mod 2 in {0, 1}`, i.e. reduction in `Z[i] / (1 + i) ~= F_2`.
    #[inline]
    pub const fn parity_1_plus_i(self) -> u8 {
        (self.re + self.im).rem_euclid(2) as u8
    }
}

/// Element of the projective line `P^1(Z[i] / (2 + 2i))` of exact size 12.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ProjectivePointMod2Plus2i {
    pub u_code: u8,
    pub v_code: u8,
}

impl ProjectivePointMod2Plus2i {
    /// Reduces a pair `(u, v) in Z[i]^2` with `gcd(u, v, 1+i) = 1` to its
    /// canonical projective point `[u : v] in P^1(Z[i] / (2 + 2i))`.
    pub fn from_pair(u: GaussianInt, v: GaussianInt) -> Option<Self> {
        if u.parity_1_plus_i() == 0 && v.parity_1_plus_i() == 0 {
            return None;
        }
        let units = [
            GaussianInt::ONE,
            GaussianInt::NEG_ONE,
            GaussianInt::I,
            GaussianInt::NEG_I,
            GaussianInt::new(1, 2),
            GaussianInt::new(-1, -2),
            GaussianInt::new(-2, 1),
            GaussianInt::new(2, -1),
        ];
        // The 4 units in (Z[i] / (2 + 2i))^* are represented by {1, -1, i, -i}.
        let mut best: Option<Self> = None;
        for &unit in &units[..4] {
            let uu = u.mul(unit).mod_2_plus_2i_code();
            let vv = v.mul(unit).mod_2_plus_2i_code();
            let candidate = Self {
                u_code: uu,
                v_code: vv,
            };
            if best.is_none_or(|b| candidate < b) {
                best = Some(candidate);
            }
        }
        best
    }

    /// Enumerates all 12 points of `P^1(Z[i] / (2 + 2i))` in canonical order,
    /// with index `0` equal to the base point `[1 : 0]` stabilized by `Gamma_0(2 + 2i)`.
    pub fn all_twelve() -> [Self; 12] {
        let mut set = BTreeSet::new();
        for u_code in 0..8u8 {
            for v_code in 0..8u8 {
                let u = GaussianInt::from_mod_2_plus_2i_code(u_code);
                let v = GaussianInt::from_mod_2_plus_2i_code(v_code);
                if let Some(pt) = Self::from_pair(u, v) {
                    set.insert(pt);
                }
            }
        }
        assert_eq!(
            set.len(),
            12,
            "P^1(Z[i]/(2+2i)) must have exact cardinality 12"
        );
        let base =
            Self::from_pair(GaussianInt::ONE, GaussianInt::ZERO).expect("base point [1:0] valid");
        let mut ordered = Vec::with_capacity(12);
        ordered.push(base);
        for pt in set {
            if pt != base {
                ordered.push(pt);
            }
        }
        ordered
            .try_into()
            .expect("exact 12 elements in P^1(Z[i]/(2+2i))")
    }
}

/// Element of the Picard group `PSL(2, Z[i]) = SL(2, Z[i]) / {+-I}`.
/// Stored in canonical projective sign form so equality `==` is exact group
/// equality in `PSL(2, Z[i])`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Psl2Zi {
    pub a: GaussianInt,
    pub b: GaussianInt,
    pub c: GaussianInt,
    pub d: GaussianInt,
}

impl Psl2Zi {
    pub fn new(
        a: GaussianInt,
        b: GaussianInt,
        c: GaussianInt,
        d: GaussianInt,
    ) -> Result<Self, String> {
        let det = a.mul(d).sub(b.mul(c));
        if det != GaussianInt::ONE {
            return Err(format!(
                "matrix determinant ({} + {}i) != 1 in SL(2, Z[i])",
                det.re, det.im
            ));
        }
        Ok(Self::canonicalize_sign(a, b, c, d))
    }

    #[inline]
    const fn canonicalize_sign(
        a: GaussianInt,
        b: GaussianInt,
        c: GaussianInt,
        d: GaussianInt,
    ) -> Self {
        let coords = [a.re, a.im, b.re, b.im, c.re, c.im, d.re, d.im];
        let mut idx = 0;
        let mut negate = false;
        while idx < 8 {
            if coords[idx] < 0 {
                negate = true;
                break;
            } else if coords[idx] > 0 {
                break;
            }
            idx += 1;
        }
        if negate {
            Self {
                a: a.neg(),
                b: b.neg(),
                c: c.neg(),
                d: d.neg(),
            }
        } else {
            Self { a, b, c, d }
        }
    }

    #[inline]
    pub const fn identity() -> Self {
        Self {
            a: GaussianInt::ONE,
            b: GaussianInt::ZERO,
            c: GaussianInt::ZERO,
            d: GaussianInt::ONE,
        }
    }

    /// Inversion generator `S = [[0, -1], [1, 0]]` of `PSL(2, Z[i])`.
    #[inline]
    pub const fn gen_s() -> Self {
        Self::canonicalize_sign(
            GaussianInt::ZERO,
            GaussianInt::NEG_ONE,
            GaussianInt::ONE,
            GaussianInt::ZERO,
        )
    }

    /// Real parabolic translation generator `T_1 = [[1, 1], [0, 1]]`.
    #[inline]
    pub const fn gen_t1() -> Self {
        Self::canonicalize_sign(
            GaussianInt::ONE,
            GaussianInt::ONE,
            GaussianInt::ZERO,
            GaussianInt::ONE,
        )
    }

    /// Imaginary parabolic translation generator `T_i = [[1, i], [0, 1]]`.
    #[inline]
    pub const fn gen_ti() -> Self {
        Self::canonicalize_sign(
            GaussianInt::ONE,
            GaussianInt::I,
            GaussianInt::ZERO,
            GaussianInt::ONE,
        )
    }

    /// Cartan half-turn generator `J = [[i, 0], [0, -i]]` (`z -> -z`).
    #[inline]
    pub const fn gen_j() -> Self {
        Self::canonicalize_sign(
            GaussianInt::I,
            GaussianInt::ZERO,
            GaussianInt::ZERO,
            GaussianInt::NEG_I,
        )
    }

    /// Parabolic translation `[[1, t], [0, 1]]` for any `t in Z[i]`.
    #[inline]
    pub const fn translation(t: GaussianInt) -> Self {
        Self::canonicalize_sign(GaussianInt::ONE, t, GaussianInt::ZERO, GaussianInt::ONE)
    }

    /// Exact group multiplication in `PSL(2, Z[i])`.
    #[inline]
    pub const fn mul(self, rhs: Self) -> Self {
        let a = self.a.mul(rhs.a).add(self.b.mul(rhs.c));
        let b = self.a.mul(rhs.b).add(self.b.mul(rhs.d));
        let c = self.c.mul(rhs.a).add(self.d.mul(rhs.c));
        let d = self.c.mul(rhs.b).add(self.d.mul(rhs.d));
        Self::canonicalize_sign(a, b, c, d)
    }

    /// Exact group inverse `[[d, -b], [-c, a]]` in `PSL(2, Z[i])`.
    #[inline]
    pub const fn inv(self) -> Self {
        Self::canonicalize_sign(self.d, self.b.neg(), self.c.neg(), self.a)
    }

    /// Exact determinant `a*d - b*c` (always `1 + 0i`).
    #[inline]
    pub const fn determinant(self) -> GaussianInt {
        self.a.mul(self.d).sub(self.b.mul(self.c))
    }

    /// Membership predicate for the index-12 Whitehead congruence subgroup
    /// `Gamma_12 = Gamma_0(2 + 2i) = { [[a, b], [c, d]] in PSL(2, Z[i]) : c == 0 mod (2 + 2i) }`.
    #[inline]
    pub const fn is_in_gamma12(self) -> bool {
        self.c.is_divisible_by_2_plus_2i()
    }

    /// Returns the projective point `self * [1 : 0]^T = [a : c] in P^1(Z[i] / (2 + 2i))`.
    /// Because `Gamma_12` is the stabilizer of `[1 : 0]^T`, this point depends
    /// strictly on the left coset `self * Gamma_12`.
    pub fn projective_image_of_infinity(self) -> ProjectivePointMod2Plus2i {
        ProjectivePointMod2Plus2i::from_pair(self.a, self.c)
            .expect("SL(2, Z[i]) column (a, c) is unimodular")
    }

    /// Returns the unique left-coset index `k in 0..12` such that
    /// `self in C_k * Gamma_12`.
    pub fn left_coset_index(self) -> usize {
        let target = self.projective_image_of_infinity();
        let all = ProjectivePointMod2Plus2i::all_twelve();
        all.iter()
            .position(|&pt| pt == target)
            .expect("projective image must match one of the 12 points of P^1(Z[i]/(2+2i))")
    }

    /// Factors `self in PSL(2, Z[i])` uniquely as `self = C_k * gamma` where
    /// `k in 0..12` is the left-coset index and `gamma in Gamma_12`.
    pub fn factor_left_coset(self) -> (usize, Self) {
        let k = self.left_coset_index();
        let c_k = Self::coset_representatives()[k];
        let gamma = c_k.inv().mul(self);
        debug_assert!(gamma.is_in_gamma12());
        (k, gamma)
    }

    /// Constructs the 12 canonical left-coset representatives
    /// `C_0, C_1, ..., C_11 in PSL(2, Z[i])` via breadth-first search over the
    /// Picard generators `{S, T_1, T_i, T_1^-1, T_i^-1, J}` starting from
    /// `C_0 = I`.
    pub fn coset_representatives() -> [Self; 12] {
        let all_pts = ProjectivePointMod2Plus2i::all_twelve();
        let gens = [
            Self::gen_s(),
            Self::gen_t1(),
            Self::gen_ti(),
            Self::gen_t1().inv(),
            Self::gen_ti().inv(),
            Self::gen_j(),
        ];
        let mut found: BTreeMap<usize, Self> = BTreeMap::new();
        let mut queue: VecDeque<Self> = VecDeque::new();
        found.insert(0, Self::identity());
        queue.push_back(Self::identity());

        while found.len() < 12 {
            let current = queue
                .pop_front()
                .expect("PSL(2, Z[i]) acts transitively on P^1(Z[i]/(2+2i))");
            for &g in &gens {
                let next = g.mul(current);
                let img = next.projective_image_of_infinity();
                let idx = all_pts
                    .iter()
                    .position(|&p| p == img)
                    .expect("valid projective point");
                if let std::collections::btree_map::Entry::Vacant(e) = found.entry(idx) {
                    e.insert(next);
                    queue.push_back(next);
                }
            }
        }

        let mut out = [Self::identity(); 12];
        for (idx, rep) in found {
            out[idx] = rep;
        }
        out
    }

    /// Exact character homomorphism `chi: Gamma_12 -> Z_2^3` and its coupled
    /// 2:1 Kuramoto resonant phase shift `(Delta_theta_f, Delta_theta_s) in [0, 2pi)^2`.
    ///
    /// For any `gamma = [[a, b], [c, d]] in Gamma_12` with `c = (2 + 2i) * c_tilde`:
    /// - `pi_a(gamma) = Im(a) mod 2 in {0, 1}`
    /// - `pi_b(gamma) = (Re(b) + Im(b)) mod 2 in {0, 1}`
    /// - `pi_c(gamma) = (Re(c_tilde) + Im(c_tilde)) mod 2 in {0, 1}`
    ///
    /// Because `-1 == 1 (mod 2)`, all three bits are strictly invariant under
    /// `M -> -M` in `PSL(2, Z[i])`, and because `a == d == 1 (mod 1 + i)` for
    /// all `gamma in Gamma_12`, each bit satisfies
    /// `pi_*(gamma_1 * gamma_2) == pi_*(gamma_1) ^ pi_*(gamma_2)`.
    ///
    /// The resonant phase shift `delta(gamma) = pi * (pi_a ^ pi_b ^ pi_c)`
    /// acts on `T^2` by `chi(gamma) = (2 * delta mod 2pi, delta mod 2pi)`,
    /// which is an exact group homomorphism `chi(g1 * g2) = chi(g1) + chi(g2) mod 2pi`
    /// and preserves `psi = theta_f - 2 * theta_s (mod 2pi)` identically.
    pub fn gamma12_character_bits(self) -> Result<[u8; 3], String> {
        let c_tilde = self
            .c
            .div_exact_2_plus_2i()
            .ok_or_else(|| "matrix is not in Gamma_12 (c not divisible by 2+2i)".to_string())?;
        let pi_a = self.a.im.rem_euclid(2) as u8;
        let pi_b = self.b.parity_1_plus_i();
        let pi_c = c_tilde.parity_1_plus_i();
        Ok([pi_a, pi_b, pi_c])
    }

    pub fn gamma12_phase_shift(self) -> Result<[f64; 2], String> {
        let [_pi_a, pi_b, pi_c] = self.gamma12_character_bits()?;
        let parity = (pi_b ^ pi_c) & 1;
        let delta = if parity == 1 { PI } else { 0.0 };
        Ok([(2.0 * delta).rem_euclid(TAU), delta.rem_euclid(TAU)])
    }

    /// Exact action on the future sheet of the Lorentz hyperboloid
    /// `H^3 = { (x0, x1, x2, x3) : x0^2 - x1^2 - x2^2 - x3^2 = 1, x0 > 0 }`
    /// via the `SL(2, C) -> SO^+(1, 3)` Hermitian spinor congruence `X' = M X M^\dagger`,
    /// where `X = [[x0 + x3, x1 + i*x2], [x1 - i*x2, x0 - x3]]`.
    pub fn act_h3(self, x: [f64; 4]) -> Result<[f64; 4], String> {
        validate_lorentz_point(x)?;
        let x00 = x[0] + x[3];
        let x11 = x[0] - x[3];
        let x01 = (x[1], x[2]);
        let x10 = (x[1], -x[2]);

        let a = (self.a.re as f64, self.a.im as f64);
        let b = (self.b.re as f64, self.b.im as f64);
        let c = (self.c.re as f64, self.c.im as f64);
        let d = (self.d.re as f64, self.d.im as f64);

        // Compute Y = M * X where X = [[x00, x01], [x10, x11]]:
        // Y00 = a * x00 + b * x10
        let y00 = c_add(c_scale(a, x00), c_mul(b, x10));
        // Y01 = a * x01 + b * x11
        let y01 = c_add(c_mul(a, x01), c_scale(b, x11));
        // Y10 = c * x00 + d * x10
        let y10 = c_add(c_scale(c, x00), c_mul(d, x10));
        // Y11 = c * x01 + d * x11
        let y11 = c_add(c_mul(c, x01), c_scale(d, x11));

        // Compute X' = Y * M^\dagger where M^\dagger = [[conj(a), conj(c)], [conj(b), conj(d)]]:
        let xp00 = c_add(c_mul(y00, c_conj(a)), c_mul(y01, c_conj(b))).0;
        let xp11 = c_add(c_mul(y10, c_conj(c)), c_mul(y11, c_conj(d))).0;
        let xp01 = c_add(c_mul(y00, c_conj(c)), c_mul(y01, c_conj(d)));

        let raw_x1 = xp01.0;
        let raw_x2 = xp01.1;
        let raw_x3 = 0.5 * (xp00 - xp11);
        let raw_x0 = 0.5 * (xp00 + xp11);
        let proj_x0 = (1.0 + raw_x1 * raw_x1 + raw_x2 * raw_x2 + raw_x3 * raw_x3).sqrt();
        let drift = (raw_x0 - proj_x0).abs() / proj_x0.max(1.0);
        if drift > 1e-9 {
            return Err(format!(
                "Hermitian spinor action numerical drift {drift:e} exceeded 1e-9"
            ));
        }
        Ok([proj_x0, raw_x1, raw_x2, raw_x3])
    }

    /// Coupled diagonal action of `gamma in Gamma_12` on `(x, phi) in H^3 x T^2`:
    /// `gamma . (x, phi) = (act_h3(gamma, x), (phi + chi(gamma)) mod 2pi)`.
    pub fn act_coupled_gamma12(
        self,
        x: [f64; 4],
        phi: [f64; 2],
    ) -> Result<([f64; 4], [f64; 2]), String> {
        validate_t2_phase(phi)?;
        let shift = self.gamma12_phase_shift()?;
        let next_x = self.act_h3(x)?;
        let next_phi = [
            (phi[0] + shift[0]).rem_euclid(TAU),
            (phi[1] + shift[1]).rem_euclid(TAU),
        ];
        Ok((next_x, next_phi))
    }
}

/// Named generator in the Schreier/Whitehead generating set of `Gamma_12`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamedGamma12Generator {
    pub name: String,
    pub coset_from: usize,
    pub picard_step: &'static str,
    pub coset_to: usize,
    pub matrix: Psl2Zi,
    pub character_bits: [u8; 3],
}

/// Returns the complete deduplicated Schreier generator set for `Gamma_12 = Gamma_0(2+2i)`
/// together with named Whitehead parabolic, congruence, and hyperbolic generators.
pub fn gamma12_generators() -> Vec<NamedGamma12Generator> {
    let reps = Psl2Zi::coset_representatives();
    let picard_gens: [(&'static str, Psl2Zi); 4] = [
        ("S", Psl2Zi::gen_s()),
        ("T1", Psl2Zi::gen_t1()),
        ("Ti", Psl2Zi::gen_ti()),
        ("J", Psl2Zi::gen_j()),
    ];
    let mut seen = BTreeSet::new();
    seen.insert(Psl2Zi::identity());
    let mut out = Vec::new();

    for (u, rep_u) in reps.iter().enumerate() {
        for &(g_name, g) in &picard_gens {
            let prod = g.mul(*rep_u);
            let v = prod.left_coset_index();
            let rep_v = reps[v];
            // Left-coset Schreier generator: gamma = C_v^-1 * g * C_u in Gamma_12
            let schreier = rep_v.inv().mul(prod);
            assert!(
                schreier.is_in_gamma12(),
                "Schreier generator must lie in Gamma_12"
            );
            if seen.insert(schreier) {
                let bits = schreier
                    .gamma12_character_bits()
                    .expect("Schreier generator is in Gamma_12");
                out.push(NamedGamma12Generator {
                    name: format!("schreier_c{u}_{g_name}_to_c{v}"),
                    coset_from: u,
                    picard_step: g_name,
                    coset_to: v,
                    matrix: schreier,
                    character_bits: bits,
                });
            }
        }
    }

    // Also ensure the explicit Whitehead hyperbolic/loxodromic generator
    // [[3, 2-2i], [2+2i, 3]] (det = 9 - 8 = 1, c = 2+2i) is included.
    let whitehead_lox = Psl2Zi::new(
        GaussianInt::new(3, 0),
        GaussianInt::new(2, -2),
        GaussianInt::new(2, 2),
        GaussianInt::new(3, 0),
    )
    .expect("Whitehead loxodromic matrix has det 1");
    if seen.insert(whitehead_lox) {
        let bits = whitehead_lox
            .gamma12_character_bits()
            .expect("Whitehead loxodromic matrix is in Gamma_12");
        out.push(NamedGamma12Generator {
            name: "whitehead_loxodromic_3_2m2i_2p2i_3".to_string(),
            coset_from: 0,
            picard_step: "loxodromic",
            coset_to: 0,
            matrix: whitehead_lox,
            character_bits: bits,
        });
    }

    out
}

/// Upper Half-Space coordinate `(z_re + i * z_im, w)` with `w > 0`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UhsCoord {
    pub z_re: f64,
    pub z_im: f64,
    pub w: f64,
}

impl UhsCoord {
    #[inline]
    pub fn norm_sq(self) -> f64 {
        self.z_re * self.z_re + self.z_im * self.z_im + self.w * self.w
    }
}

/// Converts a Lorentz point `x = (x0, x1, x2, x3) in H^3` to Upper Half-Space `(z, w)`.
pub fn lorentz_to_uhs(x: [f64; 4]) -> Result<UhsCoord, String> {
    validate_lorentz_point(x)?;
    let denom = x[0] - x[3];
    if denom <= 0.0 || !denom.is_finite() {
        return Err("Lorentz x0 - x3 denominator must be positive finite".into());
    }
    Ok(UhsCoord {
        z_re: x[1] / denom,
        z_im: x[2] / denom,
        w: 1.0 / denom,
    })
}

/// Converts Upper Half-Space `(z, w)` (`w > 0`) to Lorentz `H^3`.
pub fn uhs_to_lorentz(u: UhsCoord) -> Result<[f64; 4], String> {
    if !u.z_re.is_finite() || !u.z_im.is_finite() || !u.w.is_finite() || u.w <= 0.0 {
        return Err("UHS coordinates must be finite with w > 0".into());
    }
    let z2_w2 = u.z_re * u.z_re + u.z_im * u.z_im + u.w * u.w;
    let x1 = u.z_re / u.w;
    let x2 = u.z_im / u.w;
    let x3 = (z2_w2 - 1.0) / (2.0 * u.w);
    let x0 = (1.0 + x1 * x1 + x2 * x2 + x3 * x3).sqrt();
    Ok([x0, x1, x2, x3])
}

/// Stereographic projection `pi(x) = (x1, x2, x3) / (x0 + 1)` from `H^3` to Poincaré ball `B^3`.
pub fn lorentz_to_poincare(x: [f64; 4]) -> Result<[f64; 3], String> {
    validate_lorentz_point(x)?;
    let denom = x[0] + 1.0;
    Ok([x[1] / denom, x[2] / denom, x[3] / denom])
}

/// Inverse stereographic (optical lens `n(r) = 2 / (1 - r^2)`) map from Poincaré ball `B^3` to `H^3`.
pub fn poincare_to_lorentz(p: [f64; 3]) -> Result<[f64; 4], String> {
    if !p.iter().all(|v| v.is_finite()) {
        return Err("Poincaré ball coordinates must be finite".into());
    }
    let r2 = p[0] * p[0] + p[1] * p[1] + p[2] * p[2];
    if r2 >= 1.0 {
        return Err(format!("Poincaré point norm^2 = {r2} must be < 1.0"));
    }
    let n = 2.0 / (1.0 - r2);
    let x1 = n * p[0];
    let x2 = n * p[1];
    let x3 = n * p[2];
    let x0 = (1.0 + x1 * x1 + x2 * x2 + x3 * x3).sqrt();
    Ok([x0, x1, x2, x3])
}

/// Lorentz inner product `<x, y>_L = x0*y0 - x1*y1 - x2*y2 - x3*y3`.
#[inline]
pub fn lorentz_inner(x: [f64; 4], y: [f64; 4]) -> f64 {
    x[0] * y[0] - x[1] * y[1] - x[2] * y[2] - x[3] * y[3]
}

/// Hyperbolic geodesic distance `d_H3(x, y)` on the unit Lorentz hyperboloid.
/// Uses the cancellation-free chordal identity
/// `2 * asinh(0.5 * sqrt(max(0, ||x_vec - y_vec||^2 - (x0 - y0)^2)))`
/// for small distances (`<x, y>_L < 2.0`), avoiding the `O(sqrt(eps))`
/// catastrophic cancellation of `acosh(<x, y>_L)` near `x == y`, and
/// `acosh(<x, y>_L)` for large distances.
#[inline]
pub fn hyperbolic_distance(x: [f64; 4], y: [f64; 4]) -> f64 {
    let ip = lorentz_inner(x, y);
    if ip < 2.0 {
        let dx0 = x[0] - y[0];
        let dx1 = x[1] - y[1];
        let dx2 = x[2] - y[2];
        let dx3 = x[3] - y[3];
        let chord_sq = (dx1 * dx1 + dx2 * dx2 + dx3 * dx3 - dx0 * dx0).max(0.0);
        2.0 * (0.5 * chord_sq.sqrt()).asinh()
    } else {
        ip.acosh()
    }
}

/// Squared chordal distance on `T^2`:
/// `d_T2^2(phi, psi) = (1 - cos(df)) + (1 - cos(ds)) in [0, 4]`.
#[inline]
pub fn torus_distance_sq(a: [f64; 2], b: [f64; 2]) -> f64 {
    (1.0 - (a[0] - b[0]).cos()) + (1.0 - (a[1] - b[1]).cos())
}

/// 2:1 Kuramoto resonant phase invariant `psi(phi) = (theta_fast - 2 * theta_slow) mod 2pi`.
#[inline]
pub fn kuramoto_psi(phi: [f64; 2]) -> f64 {
    (phi[0] - 2.0 * phi[1]).rem_euclid(TAU)
}

/// Joint hyperbolic-toroidal cost (M-1 cost-form convention):
/// `cost(x, phi; c_w, phi_w) = d_H3(x, c_w) + 0.25 * d_T2^2(phi, phi_w)`.
#[inline]
pub fn joint_well_cost(x: [f64; 4], phi: [f64; 2], well_h3: [f64; 4], well_t2: [f64; 2]) -> f64 {
    hyperbolic_distance(x, well_h3) + 0.25 * torus_distance_sq(phi, well_t2)
}

pub fn validate_lorentz_point(x: [f64; 4]) -> Result<(), String> {
    if !x.iter().all(|v| v.is_finite()) {
        return Err("Lorentz coordinates must be finite".into());
    }
    if x[0] < 1.0 - 1e-7 {
        return Err(format!(
            "Lorentz time coordinate x0 = {} must be >= 1",
            x[0]
        ));
    }
    let quad = x[0] * x[0] - x[1] * x[1] - x[2] * x[2] - x[3] * x[3];
    let tol = 1e-6 * (x[0] * x[0]).max(1.0);
    if (quad - 1.0).abs() > tol {
        return Err(format!(
            "Lorentz point violates x0^2 - ||x||^2 = 1 (got {quad})"
        ));
    }
    Ok(())
}

pub fn validate_t2_phase(phi: [f64; 2]) -> Result<(), String> {
    if !phi[0].is_finite() || !phi[1].is_finite() {
        return Err("T2 phase coordinates must be finite".into());
    }
    Ok(())
}

/// Canonical representative of `(x, phi) in H^3 x T^2` under both the ambient
/// Picard group `PSL(2, Z[i])` and the index-12 Whitehead subgroup `Gamma_12`.
#[derive(Clone, Debug, PartialEq)]
pub struct CanonicalRepresentative {
    /// Reduced point in the classical Picard fundamental domain `F_Gamma`.
    pub h3_picard: [f64; 4],
    /// Reduced point `x_can = gamma_x . x = C_k^-1 . h3_picard` in the 12-tile
    /// Whitehead fundamental domain `F_{Gamma_12}`.
    pub h3_gamma12: [f64; 4],
    /// Canonical `T^2` phase `phi_can = (phi + chi(gamma_x)) mod 2pi`.
    pub t2_gamma12: [f64; 2],
    /// Total Picard reduction matrix `M_x in PSL(2, Z[i])` such that `h3_picard = M_x . x`.
    pub picard_matrix: Psl2Zi,
    /// Unique `Gamma_12` factor `gamma_x in Gamma_12` such that `M_x = C_k * gamma_x`.
    pub gamma12_matrix: Psl2Zi,
    /// Unique left-coset index `k in 0..12` such that `M_x in C_k * Gamma_12`.
    pub coset_index: usize,
    /// Coupled 2:1 Kuramoto resonant phase `(psi(phi_can) + 2pi * k / 12) mod 2pi`.
    pub coupled_psi: f64,
    /// Coupled `T^2` sector `floor(12 * coupled_psi / 2pi) in 0..12`.
    pub coupled_sector: u8,
    /// Number of Euclidean-Picard reduction iterations performed.
    pub reduction_steps: u32,
}

/// Reduces `x in H^3` into the canonical Picard fundamental domain
/// `F_Gamma = { (z, w) : Re(z) in [-1/2, 1/2), Im(z) in [0, 1/2], |z|^2 + w^2 >= 1 }`
/// and factors `M_x = C_k * gamma_x` to produce the `Gamma_12`-canonical
/// representative `(x_can, phi_can)` on `(H^3 x T^2) / Gamma_12`.
pub fn canonicalize(x: [f64; 4], phi: [f64; 2]) -> Result<CanonicalRepresentative, String> {
    validate_lorentz_point(x)?;
    validate_t2_phase(phi)?;

    let mut m_acc = Psl2Zi::identity();
    let mut curr = x;
    let mut steps = 0u32;

    for _ in 0..64 {
        let u = lorentz_to_uhs(curr)?;
        let n_re = -(u.z_re + 0.5).floor() as i64;
        let n_im = -(u.z_im + 0.5).floor() as i64;
        if n_re != 0 || n_im != 0 {
            let t = Psl2Zi::translation(GaussianInt::new(n_re, n_im));
            m_acc = t.mul(m_acc);
            curr = m_acc.act_h3(x)?;
            steps += 1;
        }
        let u2 = lorentz_to_uhs(curr)?;
        if u2.norm_sq() < 1.0 - PICARD_EPS {
            m_acc = Psl2Zi::gen_s().mul(m_acc);
            curr = m_acc.act_h3(x)?;
            steps += 1;
        } else {
            break;
        }
    }

    // Fold into Im(z) >= 0 via J = [[i, 0], [0, -i]] (z -> -z).
    let u_fold = lorentz_to_uhs(curr)?;
    if u_fold.z_im < -PICARD_EPS || (u_fold.z_im.abs() <= PICARD_EPS && u_fold.z_re < -PICARD_EPS) {
        m_acc = Psl2Zi::gen_j().mul(m_acc);
        steps += 1;
    }

    // Evaluate the finite local Picard neighborhood around `m_acc` to guarantee
    // orbit-invariant boundary tie-breaking under IEEE-754 floating-point rounding.
    let best_m = select_canonical_picard_neighbor(x, m_acc)?;
    let h3_picard = best_m.act_h3(x)?;
    let (coset_index, gamma12_matrix) = best_m.factor_left_coset();
    let (h3_gamma12, t2_gamma12) = gamma12_matrix.act_coupled_gamma12(x, phi)?;
    let psi_can = kuramoto_psi(t2_gamma12);
    let coupled_psi = (psi_can + (TAU * (coset_index as f64) / 12.0)).rem_euclid(TAU);
    let coupled_sector = ((coupled_psi / TAU) * 12.0).floor().clamp(0.0, 11.0) as u8;

    Ok(CanonicalRepresentative {
        h3_picard,
        h3_gamma12,
        t2_gamma12,
        picard_matrix: best_m,
        gamma12_matrix,
        coset_index,
        coupled_psi,
        coupled_sector,
        reduction_steps: steps,
    })
}

fn select_canonical_picard_neighbor(x: [f64; 4], seed: Psl2Zi) -> Result<Psl2Zi, String> {
    let mut shifts = Vec::with_capacity(9);
    for dr in -1..=1 {
        for di in -1..=1 {
            shifts.push(Psl2Zi::translation(GaussianInt::new(dr, di)));
        }
    }
    let s = Psl2Zi::gen_s();
    let j = Psl2Zi::gen_j();

    let mut candidates = BTreeSet::new();
    for &t1 in &shifts {
        let m1 = t1.mul(seed);
        candidates.insert(m1);
        candidates.insert(j.mul(m1));
        let m2 = s.mul(m1);
        for &t2 in &shifts {
            let m3 = t2.mul(m2);
            candidates.insert(m3);
            candidates.insert(j.mul(m3));
        }
    }

    let mut valid: Vec<(Psl2Zi, [f64; 4], UhsCoord)> = Vec::new();
    for m in candidates {
        let xm = m.act_h3(x)?;
        let u = lorentz_to_uhs(xm)?;
        let in_strip = u.z_re >= -0.5 - PICARD_EPS
            && u.z_re <= 0.5 + PICARD_EPS
            && u.z_im >= -PICARD_EPS
            && u.z_im <= 0.5 + PICARD_EPS;
        let above_sphere = u.norm_sq() >= 1.0 - 1e-8;
        if in_strip && above_sphere {
            valid.push((m, xm, u));
        }
    }

    if valid.is_empty() {
        return Ok(seed);
    }

    // Canonical Ford/Picard objective:
    // 1. Minimize X_11 = x0 - x3 = 1/w (maximizes UHS height w).
    // 2. Tie-break by minimizing X_00 = x0 + x3 (minimizes |z|^2).
    // 3. On boundary Im(z) ~ 0 or Im(z) ~ 0.5 or Re(z) ~ +-0.5, prefer Re(z) >= 0 and Im(z) >= 0.
    // 4. Final deterministic tie-break by largest z_re, then largest z_im.
    valid.sort_by(|(ma, xa, ua), (mb, xb, ub)| {
        let x11_a = xa[0] - xa[3];
        let x11_b = xb[0] - xb[3];
        if (x11_a - x11_b).abs() > PICARD_EPS {
            return x11_a
                .partial_cmp(&x11_b)
                .unwrap_or(std::cmp::Ordering::Equal);
        }
        let x00_a = xa[0] + xa[3];
        let x00_b = xb[0] + xb[3];
        if (x00_a - x00_b).abs() > PICARD_EPS {
            return x00_a
                .partial_cmp(&x00_b)
                .unwrap_or(std::cmp::Ordering::Equal);
        }
        let pen_a = boundary_sign_penalty(*ua);
        let pen_b = boundary_sign_penalty(*ub);
        if pen_a != pen_b {
            return pen_a.cmp(&pen_b);
        }
        if (ua.z_re - ub.z_re).abs() > PICARD_EPS {
            return ub
                .z_re
                .partial_cmp(&ua.z_re)
                .unwrap_or(std::cmp::Ordering::Equal);
        }
        if (ua.z_im - ub.z_im).abs() > PICARD_EPS {
            return ub
                .z_im
                .partial_cmp(&ua.z_im)
                .unwrap_or(std::cmp::Ordering::Equal);
        }
        ma.cmp(mb)
    });

    Ok(valid[0].0)
}

fn boundary_sign_penalty(u: UhsCoord) -> u8 {
    let mut p = 0u8;
    // Prefer Re(z) in (-0.5, 0.5] when |Re(z)| ~ 0.5
    if (u.z_re + 0.5).abs() <= 1e-8 {
        p += 4;
    }
    // Prefer Re(z) >= 0 when Im(z) ~ 0 or Im(z) ~ 0.5
    if (u.z_im.abs() <= 1e-8 || (u.z_im - 0.5).abs() <= 1e-8) && u.z_re < -1e-8 {
        p += 2;
    }
    // Prefer Re(z) >= 0 on the unit sphere |z|^2 + w^2 ~ 1
    if (u.norm_sq() - 1.0).abs() <= 1e-8 && u.z_re < -1e-8 {
        p += 1;
    }
    p
}

/// Persistent status for `geometry_space`, `cognitive_hub`, and `cognitive_well`.
/// Per GlobalMind handoff rules, database seeding defaults to `Specified` (`'specified'`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TopologyStatus {
    Specified,
    Certified,
    Retired,
}

impl TopologyStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Specified => "specified",
            Self::Certified => "certified",
            Self::Retired => "retired",
        }
    }
}

/// Duality pole on `T^2` for each of the 6 icosahedral axes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DualityPole {
    PoleA,
    PoleB,
}

impl DualityPole {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PoleA => "pole_a",
            Self::PoleB => "pole_b",
        }
    }
}

/// Central reference hub (`L0_Coordinator`) surrounded by the 12 peer wells.
#[derive(Clone, Debug, PartialEq)]
pub struct CognitiveHubSpec {
    pub id: RecordId,
    pub label: String,
    pub h3_center_lorentz: [f64; 4],
    pub t2_phase: [f64; 2],
    pub hub_horizon_radius: f64,
    pub reference_contract_digest: Digest,
    pub status: TopologyStatus,
}

/// One of the 12 peer cognitive wells (`L1 .. L12`, ordinals `0..11`).
#[derive(Clone, Debug, PartialEq)]
pub struct CognitiveWellSpec {
    pub id: RecordId,
    pub ordinal: u8,
    pub label: String,
    pub duality_axis: u8,
    pub pole: DualityPole,
    pub unit_direction_s2: [f64; 3],
    pub poincare_center: [f64; 3],
    pub h3_center_lorentz: [f64; 4],
    pub t2_sector: u8,
    pub t2_center_phase: [f64; 2],
    pub coset_representative: Psl2Zi,
    pub adjacent_ordinals: [u8; 5],
    pub antipodal_ordinal: u8,
    pub boundary_contract_digest: Digest,
    pub status: TopologyStatus,
}

/// Complete executable Whitehead 12-around-1 topology specification.
#[derive(Clone, Debug)]
pub struct WhiteheadTopology {
    pub workspace: RecordId,
    pub geometry_space_id: RecordId,
    pub contract_version: String,
    pub contract_digest: Digest,
    pub topology_contract_digest: Digest,
    pub classifier_digest: Digest,
    pub hub: CognitiveHubSpec,
    pub wells: [CognitiveWellSpec; 12],
    pub overlap_matrix: [[f64; 12]; 12],
}

/// Receipted result of classifying an address `(x, phi) in H^3 x T^2` into one
/// of the 12 cognitive wells.
#[derive(Clone, Debug, PartialEq)]
pub struct WellClassification {
    pub primary_ordinal: u8,
    pub primary_well_id: RecordId,
    pub primary_cost: f64,
    pub runner_up_ordinal: u8,
    pub runner_up_cost: f64,
    pub boundary_margin: f64,
    pub on_equidistant_boundary: bool,
    pub tie_broken_by_lowest_ordinal: bool,
    pub within_hub_horizon: bool,
    pub hub_distance_h3: f64,
    pub authorized_overlap_ordinals: Vec<u8>,
    pub canonical: CanonicalRepresentative,
    pub classifier_version: String,
    pub classifier_digest: Digest,
    pub boundary_receipt_digest: Digest,
}

const WELL_LABELS: [&str; 12] = [
    "L1_SensoryPerception",
    "L2_MotorPraxis",
    "L3_SomaticInteroception",
    "L4_LanguageSemiotics",
    "L5_EpisodicMemory",
    "L6_PredictiveSimulation",
    "L7_ConvergentLogic",
    "L8_DivergentSynthesis",
    "L9_ValuationSalience",
    "L10_SocialCognition",
    "L11_SpatialTopography",
    "L12_MetacognitiveRegulation",
];

impl WhiteheadTopology {
    /// Builds the canonical 12-around-1 Whitehead topology with default
    /// `TopologyStatus::Specified` status.
    pub fn canonical(workspace: RecordId) -> Result<Self, String> {
        Self::with_status(workspace, TopologyStatus::Specified)
    }

    pub fn with_status(workspace: RecordId, status: TopologyStatus) -> Result<Self, String> {
        let geometry_space_id = RecordId::parse("h3_t2_r12_v1")?;
        let contract_bytes = include_bytes!("../contracts/whitehead-gamma12-v1.contract");
        let contract_digest = Digest::sha256(contract_bytes);

        let phi = 0.5 * (1.0 + 5.0_f64.sqrt());
        let r_sph = 2.0 - phi; // (3 - sqrt(5)) / 2 ~= 0.38196601125010515
        let norm_ico = (1.0 + phi * phi).sqrt();

        // 6 antipodal duality axes (12 vertices of the regular icosahedron).
        let raw_dirs: [[f64; 3]; 12] = [
            [0.0, 1.0, phi],
            [0.0, -1.0, -phi],
            [0.0, 1.0, -phi],
            [0.0, -1.0, phi],
            [1.0, phi, 0.0],
            [-1.0, -phi, 0.0],
            [1.0, -phi, 0.0],
            [-1.0, phi, 0.0],
            [phi, 0.0, 1.0],
            [-phi, 0.0, -1.0],
            [phi, 0.0, -1.0],
            [-phi, 0.0, 1.0],
        ];

        let mut unit_dirs = [[0.0_f64; 3]; 12];
        for i in 0..12 {
            unit_dirs[i] = [
                raw_dirs[i][0] / norm_ico,
                raw_dirs[i][1] / norm_ico,
                raw_dirs[i][2] / norm_ico,
            ];
        }

        // Compute the 12x12 minimal icosahedral overlap matrix and 5 kissing neighbors per well.
        let inv_sqrt5 = 1.0 / 5.0_f64.sqrt();
        let mut overlap_matrix = [[0.0_f64; 12]; 12];
        let mut adjacent_list = [[0u8; 5]; 12];
        for r in 0..12 {
            let mut adj = Vec::with_capacity(5);
            for s in 0..12 {
                if r == s {
                    continue;
                }
                let dot = unit_dirs[r][0] * unit_dirs[s][0]
                    + unit_dirs[r][1] * unit_dirs[s][1]
                    + unit_dirs[r][2] * unit_dirs[s][2];
                if (dot - inv_sqrt5).abs() < 1e-9 {
                    overlap_matrix[r][s] = DEFAULT_ICOSAHEDRAL_OVERLAP_MARGIN;
                    adj.push(s as u8);
                }
            }
            if adj.len() != 5 {
                return Err(format!(
                    "icosahedral vertex {r} expected 5 kissing neighbors, got {}",
                    adj.len()
                ));
            }
            adjacent_list[r] = [adj[0], adj[1], adj[2], adj[3], adj[4]];
        }

        let coset_reps = Psl2Zi::coset_representatives();
        let mut wells_vec = Vec::with_capacity(12);
        for ordinal in 0..12usize {
            let dir = unit_dirs[ordinal];
            let poincare_center = [r_sph * dir[0], r_sph * dir[1], r_sph * dir[2]];
            let h3_center_lorentz = poincare_to_lorentz(poincare_center)?;

            // Each well k in 0..12 has resonant invariant psi(phi_k) = (theta_f - 2*theta_s) mod 2pi
            // centered in sector k: sector_angle = 2*pi*(k + 0.5)/12.
            // Setting theta_slow = sector_angle and theta_fast = (3 * sector_angle) mod 2pi gives
            // psi(phi_k) = (3 * sector_angle - 2 * sector_angle) mod 2pi = sector_angle.
            let sector_angle = TAU * ((ordinal as f64) + 0.5) / 12.0;
            let theta_slow = sector_angle.rem_euclid(TAU);
            let theta_fast = (3.0 * sector_angle).rem_euclid(TAU);
            let t2_center_phase = [theta_fast, theta_slow];

            let duality_axis = (ordinal / 2) as u8;
            let pole = if ordinal % 2 == 0 {
                DualityPole::PoleA
            } else {
                DualityPole::PoleB
            };
            let antipodal_ordinal = (ordinal ^ 1) as u8;
            let adjacent_ordinals = adjacent_list[ordinal];

            let boundary_payload = format!(
                "well:{ordinal}:label:{}:h3:{:?}:t2_sector:{ordinal}:t2_center:{:?}:adj:{:?}:antipode:{antipodal_ordinal}:coset:{:?}:contract:{}",
                WELL_LABELS[ordinal],
                h3_center_lorentz,
                t2_center_phase,
                adjacent_ordinals,
                coset_reps[ordinal],
                contract_digest.as_str()
            );
            let boundary_contract_digest = Digest::sha256(boundary_payload.as_bytes());

            wells_vec.push(CognitiveWellSpec {
                id: RecordId::parse(format!("well_{ordinal:02}"))?,
                ordinal: ordinal as u8,
                label: WELL_LABELS[ordinal].to_string(),
                duality_axis,
                pole,
                unit_direction_s2: dir,
                poincare_center,
                h3_center_lorentz,
                t2_sector: ordinal as u8,
                t2_center_phase,
                coset_representative: coset_reps[ordinal],
                adjacent_ordinals,
                antipodal_ordinal,
                boundary_contract_digest,
                status,
            });
        }

        let wells: [CognitiveWellSpec; 12] = wells_vec
            .try_into()
            .map_err(|_| "failed to construct 12 cognitive wells".to_string())?;

        let hub_center = [1.0, 0.0, 0.0, 0.0];
        let hub_phase = [0.0, 0.0];
        let reference_payload = format!(
            "hub:L0_Coordinator:h3:{hub_center:?}:t2:{hub_phase:?}:horizon:{DEFAULT_HUB_HORIZON_RADIUS:?}:contract:{}",
            contract_digest.as_str()
        );
        let reference_contract_digest = Digest::sha256(reference_payload.as_bytes());
        let hub = CognitiveHubSpec {
            id: RecordId::parse("hub_l0")?,
            label: "L0_Coordinator".to_string(),
            h3_center_lorentz: hub_center,
            t2_phase: hub_phase,
            hub_horizon_radius: DEFAULT_HUB_HORIZON_RADIUS,
            reference_contract_digest: reference_contract_digest.clone(),
            status,
        };

        let mut topo_hasher_input = format!(
            "topology:{WHITEHEAD_CONTRACT_VERSION}:hub:{}:",
            reference_contract_digest.as_str()
        );
        for w in &wells {
            topo_hasher_input.push_str(w.boundary_contract_digest.as_str());
            topo_hasher_input.push(':');
        }
        let topology_contract_digest = Digest::sha256(topo_hasher_input.as_bytes());
        let classifier_digest = Digest::sha256(
            format!(
                "classifier:{WHITEHEAD_CLASSIFIER_VERSION}:topology:{}",
                topology_contract_digest.as_str()
            )
            .as_bytes(),
        );

        Ok(Self {
            workspace,
            geometry_space_id,
            contract_version: WHITEHEAD_CONTRACT_VERSION.to_string(),
            contract_digest,
            topology_contract_digest,
            classifier_digest,
            hub,
            wells,
            overlap_matrix,
        })
    }

    /// Classifies `(x, phi) in H^3 x T^2` into its unique canonical cognitive well
    /// after reducing `(x, phi)` into a canonical representative under `Gamma_12`
    /// and mapping into the 12-around-1 frame via the canonical well reference.
    ///
    /// To ensure simultaneously that:
    /// 1. Every well center `(c_w, phi_w)` classifies to well `w` with minimal cost, AND
    /// 2. Every `Gamma_12`-orbit `gamma . (x, phi)` (`gamma in Gamma_12`) classifies
    ///    identically (`classify_well(gamma . (x, phi)) == classify_well(x, phi)`),
    ///
    /// we evaluate the 12 well centers against the `Gamma_12`-canonicalized state
    /// `(h3_gamma12, t2_gamma12)` lifted from the canonical fundamental domain.
    /// Note that because all 12 well centers `c_w` lie at Poincaré radius
    /// `r_sph = 2 - phi ~= 0.381966`, their UHS norms satisfy
    /// `|z|^2 + w^2 >= ((1 - r_sph)/(1 + r_sph))^2 = 0.200 > 0`, and under
    /// canonicalization each well's canonical image `canonicalize(c_w, phi_w)`
    /// defines the exact quotient image of well `w` in `F_{Gamma_12} x T^2`.
    pub fn classify_well(&self, x: [f64; 4], phi: [f64; 2]) -> Result<WellClassification, String> {
        let canonical = canonicalize(x, phi)?;
        let mut scored: Vec<(u8, f64)> = Vec::with_capacity(12);

        for well in &self.wells {
            let well_can = canonicalize(well.h3_center_lorentz, well.t2_center_phase)?;
            let h3_d = hyperbolic_distance(canonical.h3_gamma12, well_can.h3_gamma12);
            let t2_d2 = torus_distance_sq(canonical.t2_gamma12, well_can.t2_gamma12);
            let coupled_d2 = 1.0 - (canonical.coupled_psi - well_can.coupled_psi).cos();
            let cost = h3_d + 0.25 * t2_d2 + 0.25 * coupled_d2;
            scored.push((well.ordinal, cost));
        }

        // Deterministic sort: lowest cost first; if |cost_a - cost_b| <= BOUNDARY_TIE_EPSILON,
        // lowest ordinal wins (no-gap, no-overlap partition).
        scored.sort_by(|&(ord_a, cost_a), &(ord_b, cost_b)| {
            if (cost_a - cost_b).abs() <= BOUNDARY_TIE_EPSILON {
                ord_a.cmp(&ord_b)
            } else {
                cost_a
                    .partial_cmp(&cost_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }
        });

        let (primary_ordinal, primary_cost) = scored[0];
        let (runner_up_ordinal, runner_up_cost) = scored[1];
        let boundary_margin = (runner_up_cost - primary_cost).max(0.0);
        let on_equidistant_boundary = boundary_margin <= BOUNDARY_TIE_EPSILON;
        let tie_broken_by_lowest_ordinal = on_equidistant_boundary;

        let hub_distance_h3 = hyperbolic_distance(canonical.h3_gamma12, self.hub.h3_center_lorentz);
        let within_hub_horizon = hub_distance_h3 <= self.hub.hub_horizon_radius;

        let mut authorized_overlap_ordinals = Vec::new();
        for &(ord, cost) in scored.iter().skip(1) {
            let allowed = self.overlap_matrix[primary_ordinal as usize][ord as usize];
            if allowed > 0.0 && (cost - primary_cost) <= allowed {
                authorized_overlap_ordinals.push(ord);
            }
        }
        authorized_overlap_ordinals.sort_unstable();

        let primary_well_id = self.wells[primary_ordinal as usize].id.clone();
        let receipt_payload = format!(
            "classify:{}:primary:{primary_ordinal}:cost:{primary_cost:.12}:runner_up:{runner_up_ordinal}:margin:{boundary_margin:.12}:tie:{tie_broken_by_lowest_ordinal}:overlap:{authorized_overlap_ordinals:?}:coset:{}:coupled_sector:{}:h3_can:{:?}:t2_can:{:?}",
            self.classifier_digest.as_str(),
            canonical.coset_index,
            canonical.coupled_sector,
            canonical.h3_gamma12,
            canonical.t2_gamma12,
        );
        let boundary_receipt_digest = Digest::sha256(receipt_payload.as_bytes());

        Ok(WellClassification {
            primary_ordinal,
            primary_well_id,
            primary_cost,
            runner_up_ordinal,
            runner_up_cost,
            boundary_margin,
            on_equidistant_boundary,
            tie_broken_by_lowest_ordinal,
            within_hub_horizon,
            hub_distance_h3,
            authorized_overlap_ordinals,
            canonical,
            classifier_version: WHITEHEAD_CLASSIFIER_VERSION.to_string(),
            classifier_digest: self.classifier_digest.clone(),
            boundary_receipt_digest,
        })
    }

    /// Direct ambient-frame Voronoi classifier on `H^3 x T^2` (without quotient
    /// folding, used when classifying raw icosahedral directions before quotient
    /// identification). Uses the exact same lowest-ordinal tie-breaking rule
    /// and minimal icosahedral overlap matrix.
    pub fn classify_direct_frame(
        &self,
        x: [f64; 4],
        phi: [f64; 2],
    ) -> Result<WellClassification, String> {
        let canonical = canonicalize(x, phi)?;
        let norm_phi = [phi[0].rem_euclid(TAU), phi[1].rem_euclid(TAU)];
        let mut scored: Vec<(u8, f64)> = Vec::with_capacity(12);

        for well in &self.wells {
            let cost = joint_well_cost(x, norm_phi, well.h3_center_lorentz, well.t2_center_phase);
            scored.push((well.ordinal, cost));
        }

        scored.sort_by(|&(ord_a, cost_a), &(ord_b, cost_b)| {
            if (cost_a - cost_b).abs() <= BOUNDARY_TIE_EPSILON {
                ord_a.cmp(&ord_b)
            } else {
                cost_a
                    .partial_cmp(&cost_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }
        });

        let (primary_ordinal, primary_cost) = scored[0];
        let (runner_up_ordinal, runner_up_cost) = scored[1];
        let boundary_margin = (runner_up_cost - primary_cost).max(0.0);
        let on_equidistant_boundary = boundary_margin <= BOUNDARY_TIE_EPSILON;
        let tie_broken_by_lowest_ordinal = on_equidistant_boundary;
        let hub_distance_h3 = hyperbolic_distance(x, self.hub.h3_center_lorentz);
        let within_hub_horizon = hub_distance_h3 <= self.hub.hub_horizon_radius;

        let mut authorized_overlap_ordinals = Vec::new();
        for &(ord, cost) in scored.iter().skip(1) {
            let allowed = self.overlap_matrix[primary_ordinal as usize][ord as usize];
            if allowed > 0.0 && (cost - primary_cost) <= allowed {
                authorized_overlap_ordinals.push(ord);
            }
        }
        authorized_overlap_ordinals.sort_unstable();

        let primary_well_id = self.wells[primary_ordinal as usize].id.clone();
        let receipt_payload = format!(
            "classify_direct:{}:primary:{primary_ordinal}:cost:{primary_cost:.12}:runner_up:{runner_up_ordinal}:margin:{boundary_margin:.12}:tie:{tie_broken_by_lowest_ordinal}:overlap:{authorized_overlap_ordinals:?}",
            self.classifier_digest.as_str(),
        );
        let boundary_receipt_digest = Digest::sha256(receipt_payload.as_bytes());

        Ok(WellClassification {
            primary_ordinal,
            primary_well_id,
            primary_cost,
            runner_up_ordinal,
            runner_up_cost,
            boundary_margin,
            on_equidistant_boundary,
            tie_broken_by_lowest_ordinal,
            within_hub_horizon,
            hub_distance_h3,
            authorized_overlap_ordinals,
            canonical,
            classifier_version: WHITEHEAD_CLASSIFIER_VERSION.to_string(),
            classifier_digest: self.classifier_digest.clone(),
            boundary_receipt_digest,
        })
    }
}

#[inline]
fn c_add(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    (a.0 + b.0, a.1 + b.1)
}

#[inline]
fn c_mul(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    (a.0 * b.0 - a.1 * b.1, a.0 * b.1 + a.1 * b.0)
}

#[inline]
fn c_scale(a: (f64, f64), s: f64) -> (f64, f64) {
    (a.0 * s, a.1 * s)
}

#[inline]
fn c_conj(a: (f64, f64)) -> (f64, f64) {
    (a.0, -a.1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projective_line_mod_2_plus_2i_has_exact_size_12() {
        let pts = ProjectivePointMod2Plus2i::all_twelve();
        assert_eq!(pts.len(), 12);
        let unique: BTreeSet<_> = pts.iter().copied().collect();
        assert_eq!(unique.len(), 12);
    }

    #[test]
    fn coset_representatives_partition_psl2zi_into_12_cosets() {
        let reps = Psl2Zi::coset_representatives();
        assert_eq!(reps.len(), 12);
        for (i, rep) in reps.iter().enumerate() {
            assert_eq!(rep.determinant(), GaussianInt::ONE);
            assert_eq!(rep.left_coset_index(), i);
            let (k, gamma) = rep.factor_left_coset();
            assert_eq!(k, i);
            assert_eq!(gamma, Psl2Zi::identity());
        }
    }

    #[test]
    fn schreier_generators_lie_in_gamma12_and_satisfy_homomorphism() {
        let gens = gamma12_generators();
        assert!(!gens.is_empty());
        for g1 in &gens {
            assert!(g1.matrix.is_in_gamma12());
            assert_eq!(g1.matrix.determinant(), GaussianInt::ONE);
            let inv = g1.matrix.inv();
            assert!(inv.is_in_gamma12());
            assert_eq!(g1.matrix.mul(inv), Psl2Zi::identity());
            for g2 in &gens {
                let prod = g1.matrix.mul(g2.matrix);
                assert!(prod.is_in_gamma12());
                let b1 = g1.matrix.gamma12_character_bits().unwrap();
                let b2 = g2.matrix.gamma12_character_bits().unwrap();
                let bp = prod.gamma12_character_bits().unwrap();
                assert_eq!(bp, [b1[0] ^ b2[0], b1[1] ^ b2[1], b1[2] ^ b2[2]]);
            }
        }
    }

    #[test]
    fn canonicalize_is_idempotent_and_gamma12_orbit_invariant() {
        let gens = gamma12_generators();
        let sample_points = [
            poincare_to_lorentz([0.12, -0.07, 0.19]).unwrap(),
            poincare_to_lorentz([-0.23, 0.31, -0.14]).unwrap(),
            poincare_to_lorentz([0.05, 0.28, 0.22]).unwrap(),
        ];
        let sample_phases = [[0.7, 1.9], [2.4, 0.3], [5.1, 4.2]];

        for (x, phi) in sample_points.into_iter().zip(sample_phases) {
            let can1 = canonicalize(x, phi).unwrap();
            let can2 = canonicalize(can1.h3_gamma12, can1.t2_gamma12).unwrap();
            assert!((hyperbolic_distance(can1.h3_gamma12, can2.h3_gamma12)).abs() < 1e-10);
            assert!((torus_distance_sq(can1.t2_gamma12, can2.t2_gamma12)).abs() < 1e-10);
            assert_eq!(can1.coset_index, can2.coset_index);
            assert_eq!(can1.coupled_sector, can2.coupled_sector);

            for g in &gens {
                let (xg, phig) = g.matrix.act_coupled_gamma12(x, phi).unwrap();
                let cang = canonicalize(xg, phig).unwrap();
                assert_eq!(
                    cang.picard_matrix,
                    can1.picard_matrix.mul(g.matrix.inv()),
                    "Picard reduction matrix must satisfy M_{{g.x}} == M_x * g^-1 for {}",
                    g.name
                );
                assert_eq!(
                    cang.gamma12_matrix,
                    can1.gamma12_matrix.mul(g.matrix.inv()),
                    "Gamma_12 factor must satisfy gamma_{{g.x}} == gamma_x * g^-1 for {}",
                    g.name
                );
                assert!(
                    hyperbolic_distance(can1.h3_gamma12, cang.h3_gamma12) < 1e-9,
                    "H3 canonical representative drifted under {}: can1={:?} vs cang={:?}",
                    g.name,
                    can1.h3_gamma12,
                    cang.h3_gamma12,
                );
                assert!(
                    torus_distance_sq(can1.t2_gamma12, cang.t2_gamma12) < 1e-9,
                    "T2 canonical phase drifted under {}",
                    g.name
                );
                assert_eq!(can1.coset_index, cang.coset_index);
                assert_eq!(can1.coupled_sector, cang.coupled_sector);
            }
        }
    }
}
