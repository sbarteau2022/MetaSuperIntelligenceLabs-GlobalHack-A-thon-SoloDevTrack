//! Item 5b: calibrated energy accounting + real-model quality slot.
//!
//! The current bench numbers are synthetic (PRESSURE-AUDIT finding 7: the
//! Landauer constant is approximate; finding 6: quality is a marker
//! heuristic). This module is the calibration path — every synthetic
//! number gets a measured replacement, and the ledger records which is
//! which so synthetic never masquerades as calibrated.

use serde::{Deserialize, Serialize};

/// Watt-meter calibration run: steady-state wall measurement that grounds
/// the Landauer surrogate. Owner: Stewart (hardware lane).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WattCalibration {
    pub joules_measured: f64,
    pub bits_erased: u64,
    pub duration_s: f64,
    pub steady_state: bool,
}

impl WattCalibration {
    /// Measured J/bit. Replaces LANDAUER_J_PER_BIT when steady_state.
    pub fn measured_j_per_bit(&self) -> Option<f64> {
        if self.steady_state && self.bits_erased > 0 {
            Some(self.joules_measured / self.bits_erased as f64)
        } else {
            None
        }
    }
}

/// Quality source: synthetic marker (today) vs real model execution.
/// The ledger labels every quality number with its source — no mixing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QualitySource {
    SyntheticMarker,
    RealModel { model: String, receipt: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityReading {
    pub score: f64,
    pub source: QualitySource,
}

impl QualityReading {
    pub fn is_calibrated(&self) -> bool {
        matches!(self.source, QualitySource::RealModel { .. })
    }
}

/// Twin-fingerprint runner: same content under two fingerprints must behave
/// identically (teacher-invariance). Returns divergence count (0 = pass).
/// This is the PROTOCOL.md §8 behavioral test, as a harness function.
pub fn twin_fingerprint_check<A, F>(content: &A, fp_a: &str, fp_b: &str, run: F) -> TwinResult
where
    F: Fn(&str, &A) -> String,
{
    let out_a = run(fp_a, content);
    let out_b = run(fp_b, content);
    TwinResult {
        fingerprint_a: fp_a.into(),
        fingerprint_b: fp_b.into(),
        byte_identical: out_a == out_b,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwinResult {
    pub fingerprint_a: String,
    pub fingerprint_b: String,
    pub byte_identical: bool,
}

/// HyperMap ingest receipt: bridges harness energy accounting to the live
/// map (map.ingest returns atom_id + receipt; the harness records the cost).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperMapIngest {
    pub atom_id: u64,
    pub receipt_sha256: String,
    pub joules_spent: f64,
    pub quality: QualityReading,
}
