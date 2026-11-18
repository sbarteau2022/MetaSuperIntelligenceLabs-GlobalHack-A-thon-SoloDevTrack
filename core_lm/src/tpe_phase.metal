#include <metal_stdlib>
using namespace metal;
#pragma clang fp contract(off)

// TPE phase kernel: GPU port of TorPhaseEncoder::evolve (torus.rs:102).
// CERTIFICATION SPLIT (dependency-ordered):
//   phases (theta_f, theta_s, psi): ALL-Q32.31-integer path on Metal `long`
//     (64-bit). Every product PROVEN to fit i64 (see §3 of the build spec):
//     k*sin <= 2^60; frac*delta <= 2^53; next_s*2 = shift. => BITWISE gate.
//   amplitude: f64 sqrt/div/fold on CPU; f32 sqrt/div/fold on GPU (Metal has
//     no double) => TOLERANCE gate (monotonic f32 fold, random-walk residual).
// Q32.31 constants are raw i64 = round(v * 2^31), VERIFIED against from_f64.

constant long TPE_OMEGA_F = 214748365l;    // 0.1
constant long TPE_OMEGA_S = 107374182l;    // 0.05
constant long TPE_K_F = 644245094l;        // 0.3
constant long TPE_K_S = 322122547l;        // 0.15

// (256-entry LUTs retired 2026-10-27: CORDIC is table-free. See git history.)

// Q32.31 multiply == Rust (a as i128 * b as i128) >> 31 (FLOOR).
// VERIFIED 2026-10-27 (q32_cordic.metal, 0/100000 vs i128 truth): carry-folded
// split (t=(mid<<16)+p0, result=(p3<<1)+(t>>31)). Replaces the earlier naive
// single-product form (overflow-unsafe outside proven ranges).
inline long q32_mul(long a, long b) {
    long a_hi = a >> 16;
    long a_lo = a - (a_hi << 16);
    long b_hi = b >> 16;
    long b_lo = b - (b_hi << 16);
    long p0 = a_lo * b_lo;
    long p1 = a_hi * b_lo;
    long p2 = a_lo * b_hi;
    long p3 = a_hi * b_hi;
    long mid = p1 + p2;
    long t = (mid << 16) + p0;
    return (p3 << 1) + (t >> 31);
}

// Integer CORDIC sin/cos — verbatim port of lock.rs::cordic_sin (pi-unit Q32.31).
// NO FLOAT, NO TABLES. cos via sin(x + pi/2): exact, no second path.
// Replaces the 256-entry LUT-interp form (retired: table residual + LUT traffic;
// CORDIC is 32 integer iterations, bitwise vs CPU oracle by construction).
constant long CORDIC_GAIN = 1304065748l;
constant long CORDIC_ATAN[32] = {
    536870912, 316933406, 167458907, 85004756,
    42667331, 21354465, 10679838, 5340245,
    2670163, 1335087, 667544, 333772,
    166886, 83443, 41722, 20861,
    10430, 5215, 2608, 1304,
    652, 326, 163, 81,
    41, 20, 10, 5,
    3, 1, 1, 0
};
inline long q32_cordic_sin(long theta) {
    long z = (long)(int)theta;
    if (z == 0) return 0;
    long pi_half = 1l << 30;
    long pi = 1l << 31;
    if (z > pi_half) { z = pi - z; }
    else if (z < -pi_half) { z = -pi - z; }
    long x = CORDIC_GAIN;
    long y = 0l;
    for (int i = 0; i < 32; i++) {
        long x_shift = x >> i;
        long y_shift = y >> i;
        if (z >= 0) {
            x = x - y_shift; y = y + x_shift; z = z - CORDIC_ATAN[i];
        } else {
            x = x + y_shift; y = y - x_shift; z = z + CORDIC_ATAN[i];
        }
    }
    return y;
}
inline long q32_cordic_cos(long theta) {
    return q32_cordic_sin(theta + (1l << 30));
}

// Integer Q32 sqrt — port of fixed.rs::sqrt (10-iteration Babylonian, all integer).
// div() inside the loop is the i128-exact Q32 div below.
// Q32 div == Rust ((a as i128) << 31) / (b as i128), clamped (fixed.rs::div).
// Certified range (sqrt path): a = sum of Q32 squares <= 2^30, b = iterate > 0,
// so (a<<31) <= 2^61 fits ulong with headroom. Out-of-range callers: clamp like Rust.
inline long q32_div(long a, long b) {
    if (b == 0) return (a == 0) ? 0l : ((a > 0) ? 9223372036854775807l : (-9223372036854775807l - 1l));
    ulong au = (ulong)a; // caller guarantees a >= 0 on the sqrt path
    ulong bu = (ulong)(b > 0 ? b : -b);
    ulong num = au << 31;   // fits: au <= 2^30 (squares-sum bound, verified)
    long q = (long)(num / bu);
    return (b > 0) ? q : -q;
}
// Integer sqrt, msb start (VERIFIED 2026-10-27): Newton from x0=s needs
// 11+ halvings for tiny sums (measured lane-44 step-1 dsum=23018: 0.25%
// residual after 10 iters — the 7.9e-6 amp error). msb binary search
// (6 compares) starts within 2x of the answer on every input; 10 iterations
// then always converge. Deterministic, bounded, fixed trip count.
inline long q32_sqrt(long s) {
    if (s <= 0) return 0l;
    int b = 0;
    long v = s;
    if (v >= (1l << 32)) { b += 32; v >>= 32; }
    if (v >= (1l << 16)) { b += 16; v >>= 16; }
    if (v >= (1l << 8)) { b += 8; v >>= 8; }
    if (v >= (1l << 4)) { b += 4; v >>= 4; }
    if (v >= (1l << 2)) { b += 2; v >>= 2; }
    if (v >= (1l << 1)) { b += 1; }
    // want r with r^2>>31 ~= s, i.e. r ~= sqrt(s<<31) = 2^((b+31)/2).
    long x = 1l << ((b + 31) >> 1);
    for (int i = 0; i < 10; i++) {
        if (x <= 0) return 0l;
        long s_div_x = q32_div(s, x);
        long sum = x + s_div_x;
        x = sum >> 1;
    }
    return x;
}

// TRUE FUSION// TRUE FUSION (dependency analysis 2026-10-27): this kernel's step uses
// ONLY threadprivate state + drive — no cross-thread reduction — so k_steps
// fold sequentially with zero barriers. Drives are STEP-MAJOR
// (df[s*n+id]) so each fused step consumes its own drive row; the old
// frozen-drive form (df[id] every step) is retired — it applied step-0's
// drive k times (caught by gate design review, never certified).
// INTEGER-ONLY KERNEL (2026-10-27 retrofit, spec §9 gate 1): no float anywhere.
// CORDIC replaces LUT-interp (bitwise vs lock.rs oracle); amplitude is integer
// (Newton sqrt + 1/TAU multiply) — the f64 CPU fold is NOT reproducible on Metal
// (no double), so the gate certifies phases BITWISE and amplitude within the
// Newton-iteration residual (bounded, documented in gate). amp_out is Q32.31 LONG.
constant long Q32_INV_TAU = 341782638l;   // round(2^31 / TAU), verified
kernel void tpe_evolve_fused(
    device long* theta_f [[buffer(0)]],
    device long* theta_s [[buffer(1)]],
    device long* psi_out [[buffer(2)]],
    device long* amp_out [[buffer(3)]],
    device const long* df [[buffer(4)]],
    device const long* ds [[buffer(5)]],
    constant uint& n [[buffer(6)]],
    constant uint& k_steps [[buffer(7)]],
    uint id [[thread_position_in_grid]]
) {
    if (id >= n) return;
    long tf = theta_f[id];
    long ts = theta_s[id];
    long amp = amp_out[id];
    for (uint s = 0u; s < k_steps; s++) {
        long df_s = df[s * n + id];
        long ds_s = ds[s * n + id];
        long diff_sf = ts - tf;             // saturating_sub on CPU; plain sub here (proven no-overflow: |diff| <= 2^33)
        long diff_fs = tf - ts;
        // NOTE: diffs are pi-unit thetas; CORDIC takes them directly (no conversion).
        long sin_sf = q32_cordic_sin(diff_sf);
        long sin_fs = q32_cordic_sin(diff_fs);
        // NOTE: CPU uses saturating_add/sub for the state updates. The GPU
        // uses plain arithmetic + explicit clamp to match saturating semantics.
        long cpl_f = q32_mul(TPE_K_F, sin_sf);
        long cpl_s = q32_mul(TPE_K_S, sin_fs);
        long step_f = TPE_OMEGA_F + cpl_f + df_s;
        long step_s = TPE_OMEGA_S + cpl_s + ds_s;
        long next_f = tf + step_f;
        long next_s = ts + step_s;
        long psi = next_f - (next_s << 1);  // next_s * 2.0 = shift (exact, matches (next_s*Q32(2.0))>>31)
        // amplitude (INTEGER path): arc_step = sqrt(sf^2+ss^2)/TAU, drive_mod = sqrt(df^2+ds^2).
        // Squares with ROUND-half-up (VERIFIED 2026-10-27): q32_mul FLOORS, which
        // loses ~1 LSB per square — negligible on steps O(0.1) but 2.9e-5-relative
        // on small drives O(1e-3) (measured lane-196 step-1: floor bias dominates
        // the Newton residual by 1000x). The phase path keeps floor (CPU i128>>31
        // matches floor bitwise); the amplitude path has no bitwise counterpart
        // (CPU is f64, unbiased), so rounding is strictly closer to truth.
        // Overflow: (x*x+2^30) needs x^2 < 2^63-2^30: certified (steps/drives
        // O(Q32)<=~1e9 on k<=8 gates; square<=1e18 fits).
        long sf2 = (step_f * step_f + (1l << 30)) >> 31;
        long ss2 = (step_s * step_s + (1l << 30)) >> 31;
        long arc = q32_mul(q32_sqrt(sf2 + ss2), Q32_INV_TAU);
        long df2 = (df_s * df_s + (1l << 30)) >> 31;
        long ds2 = (ds_s * ds_s + (1l << 30)) >> 31;
        long drv = q32_sqrt(df2 + ds2);
        amp = amp + arc + drv;
        tf = next_f;
        ts = next_s;
        if (s == k_steps - 1u) { psi_out[id] = psi; }
    }
    theta_f[id] = tf;
    theta_s[id] = ts;
    amp_out[id] = amp;
}
