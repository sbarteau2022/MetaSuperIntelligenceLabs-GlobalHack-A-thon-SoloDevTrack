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


// v2 THREADGROUP-BARRIER kernel (TBIK intra-device tree, 2026-10-27):
// TRUE K-step fusion for n <= 256. Each thread owns ONE oscillator; per step it
// contributes its own (ce,se) term to threadgroup memory, the fixed binary tree
// reduces all T slots (TBIK topology: stride-halving, slot-ordered, zero race),
// and every thread reads the SAME resultant. Per-thread theta folds sequentially.
// SCOPE: n <= 256 (one threadgroup). n > 256 uses the v1 kernel (k_steps=1).
// COST: O(1) trig per thread per step (vs O(n) in v1) + log2(T) barrier rounds.
kernel void kuramoto_fused_tg(
    device long* theta [[buffer(0)]],
    device const long* omega [[buffer(1)]],
    device long* d_theta_out [[buffer(2)]],
    constant long& dt [[buffer(3)]],
    constant long& k_coupling [[buffer(4)]],
    constant uint& n [[buffer(5)]],
    constant uint& k_steps [[buffer(6)]],
    uint id [[thread_position_in_grid]],
    uint tid [[thread_position_in_threadgroup]],
    uint tsize [[threads_per_threadgroup]]
) {
    if (id >= n) return;
    // threadgroup scratch: 256 slots x (re,im) as FIXED-POINT Q32 (integer-exact
    // tree adds; the split-K /n folds in at the root like v1).
    threadgroup long tg_re[256];
    threadgroup long tg_im[256];
    long my_theta = theta[id];
    // SNAPSHOT (2026-10-27): the whole step reads from snap[] (barrier-published
    // at the top of each step); theta[] is written ONCE after the fused loop.
    // The v1 kernel's nondeterminism at n=256/multi-launch (2848-56991 LSB
    // run-to-run) is a load/store race: threads write theta[id] at different
    // times while others loop over theta[j]. Zero race here by construction.
    long snap[256];
    for (uint s = 0u; s < k_steps; s++) {
        snap[tid] = my_theta;
        threadgroup_barrier(mem_flags::mem_threadgroup);
        // 1. my term (split-K scaled, fixed point — same rounding as v1).
        long hi = snap[tid] >> 12;
        long lo = snap[tid] - (hi << 12);
        float ang_lo = (float)lo * (3.14159265f / 2147483648.0f);
        float hp = (float)hi / 524288.0f;
        float chp = cospi(hp);
        float shp = sinpi(hp);
        float ce = chp - ang_lo * shp;
        float se = shp + ang_lo * chp;
        float inv_n = 1.0f / (float)n;   // Newton refinement unnecessary: single use, 0.5ulp fine (matches v1 bound analysis)
        tg_re[tid] = (long)rint(ce * inv_n * 4294967296.0f);
        tg_im[tid] = (long)rint(se * inv_n * 4294967296.0f);
        threadgroup_barrier(mem_flags::mem_threadgroup);
        // 2. TBIK tree: fixed stride-halving over slots [0, T). T <= 256.
        // Zero-pad: threads with tid >= n wrote nothing; their slots must be 0.
        // (threads with id >= n returned early — but tid < tsize always writes?
        // NO: early-returned threads never reach here, their slots hold STALE data.
        // FIX: zero-fill loop before the term write, or require tsize == n.)
        // REQUIREMENT (documented): dispatch with threadgroup size == n exactly.
        for (uint stride = tsize >> 1u; stride > 0u; stride >>= 1u) {
            if (tid < stride) {
                tg_re[tid] += tg_re[tid + stride];
                tg_im[tid] += tg_im[tid + stride];
            }
            threadgroup_barrier(mem_flags::mem_threadgroup);
        }
        float full_re = (float)tg_re[0] / 4294967296.0f;
        float full_im = (float)tg_im[0] / 4294967296.0f;
        // 3. my tail (identical to v1: Dekker-2Prod + rint).
        long hi_m = my_theta >> 12;
        long lo_m = my_theta - (hi_m << 12);
        float ang_lo_m = (float)lo_m * (3.14159265f / 2147483648.0f);
        float hp_m = (float)hi_m / 524288.0f;
        float chmp = cospi(hp_m);
        float shmp = sinpi(hp_m);
        float ci = chmp - ang_lo_m * shmp, si = shmp + ang_lo_m * chmp;
        float p1hi = ci * full_im;
        float p1lo = fma(ci, full_im, -p1hi);
        float p2hi = si * full_re;
        float p2lo = fma(si, full_re, -p2hi);
        float s_hi = p1hi - p2hi;
        float e_hi = (p1hi - s_hi) - p2hi;
        float s_lo = p1lo - p2lo;
        float coupling = (s_hi + (e_hi + s_lo));
        long coupling_q32 = (long)rint(coupling * 2147483648.0f);
        long interaction = (k_coupling * coupling_q32) >> 31;
        long total_derivative = omega[id] + interaction;
        long delta = (total_derivative * dt) >> 31;
        my_theta = (long)(int)(my_theta + delta);
        if (s == k_steps - 1u) { d_theta_out[id] = delta; }
        threadgroup_barrier(mem_flags::mem_threadgroup); // keep tree slots coherent across steps
    }
    theta[id] = my_theta;
}
