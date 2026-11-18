#include <metal_stdlib>
using namespace metal;
#pragma clang fp contract(off)

// PATCH-03 GPU side: mean-field Kuramoto evolution, fused K steps.
// Companion to kuramoto_tile.metal (all-pairs O(n^2)) and lock.rs CPU oracle.
// State stays fixed-point pi-unit Q32_31. Coupling math identical to oracle.

// NOTE (2026-10-27): the LUT path is RETIRED. The sinpi/cospi no-epicycles
// path (hp = hi/524288 exact + ang_lo first-order) dominates it on accuracy
// (terms <= 0.5ulp VERIFIED vs LUT 12-bit quantization ~12k-LSB residual)
// and needs no table traffic. LUT retained only in sin_lut_4096.inc for
// reference; this kernel takes no LUT dependency.

kernel void kuramoto_meanfield_fused(
    device long* theta [[buffer(0)]],
    device const long* omega [[buffer(1)]],
    device long* d_theta_out [[buffer(2)]],
    constant long& dt [[buffer(3)]],
    constant long& k_coupling [[buffer(4)]],
    constant uint& n [[buffer(5)]],
    constant uint& k_steps [[buffer(6)]],
    uint id [[thread_position_in_grid]],
    uint grid_size [[threads_per_grid]],
    uint tid [[thread_position_in_threadgroup]],
    uint tsize [[threads_per_threadgroup]]
) {
    if (id >= n) return;
    long my_theta = theta[id];
    // RACE-FREE SNAPSHOT (2026-10-27): the s-loop reads theta[j] live from
    // global memory while other threads write theta[id] — measured 2848-56991
    // LSB run-to-run nondeterminism at n=256 sequential launches. For n <= 256
    // (all certified configs: single threadgroup) snapshot once into
    // threadgroup memory behind a barrier; the loop then reads the snapshot.
    // n > 256 spans threadgroups (no cross-group visibility): falls back to
    // the live global read with identical numerics but launch-skew sensitivity
    // (documented; the certified gate pins n <= 256).
    threadgroup long snap[256];
    bool use_snap = (n <= 256u);
    if (use_snap) {
        if (tid < n) { snap[tid] = theta[tid]; }
        threadgroup_barrier(mem_flags::mem_threadgroup);
    }
    // Integer-exact phase path (mantissa-theorem compliant):
    // theta = hi*4096 + lo split; hp = hi/524288 EXACT in fp32
    // (|hi| <= 2^20 < 2^24); cospi/sinpi do their own exact reduction;
    // ang_lo is the only first-order correction. contract(off) keeps the
    // Dekker residuals live (VERIFIED: Kahan/Neumaier/2Sum all contract
    // to naive without it — V1==V2==V3 bit-identical).
    // FROZEN-RESULTANT NOTE (2026-10-27, measured 48M LSB @k=8): this loop
    // re-reads step-0 theta every step (no cross-thread visibility
    // mid-dispatch), so k_steps>1 is K repeats of the step-0 resultant — an
    // explicit-Euler-with-frozen-coupling integrator, NOT K sequential steps.
    // TRUE fusion lives in kuramoto_fused_tg.metal (TBIK intra-device tree,
    // self_max=0 vs k sequential launches — CERTIFIED). The k_steps=1 path
    // here is CERTIFIED (gate); k_steps>1 is a documented approximation for
    // amortizing dispatch, uncertified vs the oracle.
    for (uint s = 0u; s < k_steps; s++) {
        // Split-K accumulation: K = (float)n exact (n<=2560 < 2^24);
        // each thread sums terms/n in fixed point, so acc NEVER exceeds
        // O(1) and the (float)(acc)/2^32 requant loses 0.5ulp of O(1)
        // (~6e-8) instead of 0.5ulp of O(n) (7.6e-6 at n=256).
        // The /n then costs nothing (already divided).
        // inv_n: fma-based correctly-rounded reciprocal (CUDA rrcp-grade):
        // e = 1 - n*rcp; inv = rcp + e*rcp (one Newton step, all rounded).
        float rcp_n = 1.0f / (float)n;
        float inv_n = fma(-(float)n, rcp_n, 1.0f);
        inv_n = fma(inv_n, rcp_n, rcp_n);
        long acc_re = 0l, acc_im = 0l;
        for (uint j = 0u; j < n; j++) {
            long tj = use_snap ? snap[j] : theta[j];
            long hi = tj >> 12;
            long lo = tj - (hi << 12);
            float ang_lo = (float)lo * (3.14159265f / 2147483648.0f);
            float hp = (float)hi / 524288.0f;
            float chp = cospi(hp);
            float shp = sinpi(hp);
            // NOTE (2026-10-27): sincos(hp*pi) TESTED — same 3-4 LSB floor
            // (the residual is term-trig-library, not reduction count).
            // Keep cospi/sinpi: each exactly reduced, independently optimal.
            float ce = chp - ang_lo * shp;
            float se = shp + ang_lo * chp;
            acc_re += (long)rint(ce * inv_n * 4294967296.0f);
            acc_im += (long)rint(se * inv_n * 4294967296.0f);
        }
        float full_re = (float)acc_re / 4294967296.0f;
        float full_im = (float)acc_im / 4294967296.0f;
        long hi_m = my_theta >> 12;
        long lo_m = my_theta - (hi_m << 12);
        float ang_lo_m = (float)lo_m * (3.14159265f / 2147483648.0f);
        float hp_m = (float)hi_m / 524288.0f;
        float chmp = cospi(hp_m);
        float shmp = sinpi(hp_m);
        float ci = chmp - ang_lo_m * shmp, si = shmp + ang_lo_m * chmp;
        // Dekker-2Prod tail: exact product residuals via fma; the
        // hi-subtract residual e_hi == 0 throughout (PROVEN: tail exact,
        // residual gap is term trig, not tail).
        float p1hi = ci * full_im;
        float p1lo = fma(ci, full_im, -p1hi);
        float p2hi = si * full_re;
        float p2lo = fma(si, full_re, -p2hi);
        float s_hi = p1hi - p2hi;
        float e_hi = (p1hi - s_hi) - p2hi;
        float s_lo = p1lo - p2lo;
        float coupling = (s_hi + (e_hi + s_lo)); // acc holds the MEAN
        // rint (half-even); exact halves are measure-zero here.
        long coupling_q32 = (long)rint(coupling * 2147483648.0f);
        long interaction = (k_coupling * coupling_q32) >> 31;
        long total_derivative = omega[id] + interaction;
        long delta = (total_derivative * dt) >> 31;
        my_theta = (long)(int)(my_theta + delta);
        if (s == k_steps - 1u) { d_theta_out[id] = delta; }
    }
    theta[id] = my_theta;


}

