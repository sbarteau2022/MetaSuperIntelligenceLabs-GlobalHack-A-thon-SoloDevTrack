//! Q32.31 fixed-point scalar (backend-native, same semantics as the source
//! system: `i64` raw, value = raw / 2^31, `i128` widened multiply).

use std::fmt;
use std::ops::{Add, Mul, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Q32_31(pub i64);

impl Q32_31 {
    pub const FRACTIONAL_BITS: u32 = 31;
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(1i64 << Self::FRACTIONAL_BITS);
    pub const HALF: Self = Self(1i64 << (Self::FRACTIONAL_BITS - 1));

    pub fn from_f64(val: f64) -> Self {
        let scaled = (val * (1i64 << Self::FRACTIONAL_BITS) as f64).round();
        // Clamp instead of wrapping: saturation is the documented guard the
        // geometric kernel lacked; here it is part of the type.
        Self(scaled.clamp(i64::MIN as f64, i64::MAX as f64) as i64)
    }

    pub fn to_f64(self) -> f64 {
        (self.0 as f64) / (1i64 << Self::FRACTIONAL_BITS) as f64
    }
}

impl Add for Q32_31 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self(self.0.saturating_add(other.0))
    }
}

impl Sub for Q32_31 {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self(self.0.saturating_sub(other.0))
    }
}

impl Mul for Q32_31 {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        let res = (self.0 as i128 * other.0 as i128) >> Self::FRACTIONAL_BITS;
        Self(res.clamp(i64::MIN as i128, i64::MAX as i128) as i64)
    }
}

impl fmt::Display for Q32_31 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.6}", self.to_f64())
    }
}
