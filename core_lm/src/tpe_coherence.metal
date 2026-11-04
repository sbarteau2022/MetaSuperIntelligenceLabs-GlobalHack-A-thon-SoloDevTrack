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
constant long TPE_TWO_PI = 13492913325l;   // 2*PI*2^31 (fixed.rs lut_interp)
constant long TPE_PHASE_SCALE = 322122547l;   // 0.15
constant long TPE_LUT_STEP = 52706692l;    // TWO_PI/256 integer division (13492913325/256, matches CPU)

constant long TPE_SIN_LUT[256] = {
    0, 52701887, 105372028, 157978697,
    210490206, 262874923, 315101295, 367137861,
    418953276, 470516330, 521795963, 572761285,
    623381598, 673626408, 723465451, 772868706,
    821806413, 870249095, 918167572, 965532978,
    1012316784, 1058490808, 1104027237, 1148898640,
    1193077991, 1236538675, 1279254516, 1321199781,
    1362349204, 1402678000, 1442161874, 1480777044,
    1518500250, 1555308768, 1591180426, 1626093616,
    1660027308, 1692961062, 1724875040, 1755750017,
    1785567396, 1814309216, 1841958164, 1868497586,
    1893911494, 1918184581, 1941302225, 1963250501,
    1984016189, 2003586779, 2021950484, 2039096241,
    2055013723, 2069693342, 2083126254, 2095304370,
    2106220352, 2115867626, 2124240380, 2131333572,
    2137142927, 2141664948, 2144896910, 2146836866,
    2147483648, 2146836866, 2144896910, 2141664948,
    2137142927, 2131333572, 2124240380, 2115867626,
    2106220352, 2095304370, 2083126254, 2069693342,
    2055013723, 2039096241, 2021950484, 2003586779,
    1984016189, 1963250501, 1941302225, 1918184581,
    1893911494, 1868497586, 1841958164, 1814309216,
    1785567396, 1755750017, 1724875040, 1692961062,
    1660027308, 1626093616, 1591180426, 1555308768,
    1518500250, 1480777044, 1442161874, 1402678000,
    1362349204, 1321199781, 1279254516, 1236538675,
    1193077991, 1148898640, 1104027237, 1058490808,
    1012316784, 965532978, 918167572, 870249095,
    821806413, 772868706, 723465451, 673626408,
    623381598, 572761285, 521795963, 470516330,
    418953276, 367137861, 315101295, 262874923,
    210490206, 157978697, 105372028, 52701887,
    0, -52701887, -105372028, -157978697,
    -210490206, -262874923, -315101295, -367137861,
    -418953276, -470516330, -521795963, -572761285,
    -623381598, -673626408, -723465451, -772868706,
    -821806413, -870249095, -918167572, -965532978,
    -1012316784, -1058490808, -1104027237, -1148898640,
    -1193077991, -1236538675, -1279254516, -1321199781,
    -1362349204, -1402678000, -1442161874, -1480777044,
    -1518500250, -1555308768, -1591180426, -1626093616,
    -1660027308, -1692961062, -1724875040, -1755750017,
    -1785567396, -1814309216, -1841958164, -1868497586,
    -1893911494, -1918184581, -1941302225, -1963250501,
    -1984016189, -2003586779, -2021950484, -2039096241,
    -2055013723, -2069693342, -2083126254, -2095304370,
    -2106220352, -2115867626, -2124240380, -2131333572,
    -2137142927, -2141664948, -2144896910, -2146836866,
    -2147483648, -2146836866, -2144896910, -2141664948,
    -2137142927, -2131333572, -2124240380, -2115867626,
    -2106220352, -2095304370, -2083126254, -2069693342,
    -2055013723, -2039096241, -2021950484, -2003586779,
    -1984016189, -1963250501, -1941302225, -1918184581,
    -1893911494, -1868497586, -1841958164, -1814309216,
    -1785567396, -1755750017, -1724875040, -1692961062,
    -1660027308, -1626093616, -1591180426, -1555308768,
    -1518500250, -1480777044, -1442161874, -1402678000,
    -1362349204, -1321199781, -1279254516, -1236538675,
    -1193077991, -1148898640, -1104027237, -1058490808,
    -1012316784, -965532978, -918167572, -870249095,
    -821806413, -772868706, -723465451, -673626408,
    -623381598, -572761285, -521795963, -470516330,
    -418953276, -367137861, -315101295, -262874923,
    -210490206, -157978697, -105372028, -52701887
};

constant long TPE_COS_LUT[256] = {
    2147483648, 2146836866, 2144896910, 2141664948,
    2137142927, 2131333572, 2124240380, 2115867626,
    2106220352, 2095304370, 2083126254, 2069693342,
    2055013723, 2039096241, 2021950484, 2003586779,
    1984016189, 1963250501, 1941302225, 1918184581,
    1893911494, 1868497586, 1841958164, 1814309216,
    1785567396, 1755750017, 1724875040, 1692961062,
    1660027308, 1626093616, 1591180426, 1555308768,
    1518500250, 1480777044, 1442161874, 1402678000,
    1362349204, 1321199781, 1279254516, 1236538675,
    1193077991, 1148898640, 1104027237, 1058490808,
    1012316784, 965532978, 918167572, 870249095,
    821806413, 772868706, 723465451, 673626408,
    623381598, 572761285, 521795963, 470516330,
    418953276, 367137861, 315101295, 262874923,
    210490206, 157978697, 105372028, 52701887,
    0, -52701887, -105372028, -157978697,
    -210490206, -262874923, -315101295, -367137861,
    -418953276, -470516330, -521795963, -572761285,
    -623381598, -673626408, -723465451, -772868706,
    -821806413, -870249095, -918167572, -965532978,
    -1012316784, -1058490808, -1104027237, -1148898640,
    -1193077991, -1236538675, -1279254516, -1321199781,
    -1362349204, -1402678000, -1442161874, -1480777044,
    -1518500250, -1555308768, -1591180426, -1626093616,
    -1660027308, -1692961062, -1724875040, -1755750017,
    -1785567396, -1814309216, -1841958164, -1868497586,
    -1893911494, -1918184581, -1941302225, -1963250501,
    -1984016189, -2003586779, -2021950484, -2039096241,
    -2055013723, -2069693342, -2083126254, -2095304370,
    -2106220352, -2115867626, -2124240380, -2131333572,
    -2137142927, -2141664948, -2144896910, -2146836866,
    -2147483648, -2146836866, -2144896910, -2141664948,
    -2137142927, -2131333572, -2124240380, -2115867626,
    -2106220352, -2095304370, -2083126254, -2069693342,
    -2055013723, -2039096241, -2021950484, -2003586779,
    -1984016189, -1963250501, -1941302225, -1918184581,
    -1893911494, -1868497586, -1841958164, -1814309216,
    -1785567396, -1755750017, -1724875040, -1692961062,
    -1660027308, -1626093616, -1591180426, -1555308768,
    -1518500250, -1480777044, -1442161874, -1402678000,
    -1362349204, -1321199781, -1279254516, -1236538675,
    -1193077991, -1148898640, -1104027237, -1058490808,
    -1012316784, -965532978, -918167572, -870249095,
    -821806413, -772868706, -723465451, -673626408,
    -623381598, -572761285, -521795963, -470516330,
    -418953276, -367137861, -315101295, -262874923,
    -210490206, -157978697, -105372028, -52701887,
    0, 52701887, 105372028, 157978697,
    210490206, 262874923, 315101295, 367137861,
    418953276, 470516330, 521795963, 572761285,
    623381598, 673626408, 723465451, 772868706,
    821806413, 870249095, 918167572, 965532978,
    1012316784, 1058490808, 1104027237, 1148898640,
    1193077991, 1236538675, 1279254516, 1321199781,
    1362349204, 1402678000, 1442161874, 1480777044,
    1518500250, 1555308768, 1591180426, 1626093616,
    1660027308, 1692961062, 1724875040, 1755750017,
    1785567396, 1814309216, 1841958164, 1868497586,
    1893911494, 1918184581, 1941302225, 1963250501,
    1984016189, 2003586779, 2021950484, 2039096241,
    2055013723, 2069693342, 2083126254, 2095304370,
    2106220352, 2115867626, 2124240380, 2131333572,
    2137142927, 2141664948, 2144896910, 2146836866
};

// Q32.31 multiply: (a*b)>>31. PROVEN SAFE in 64 bits for the TPE path
// (k*sin <= 2^60, frac*delta <= 2^53). Saturates like Q32_31::mul.
inline long q32_mul(long a, long b) {
    long p = a * b;                        // fits i64 on the TPE path (proven)
    long r = p >> 31;                      // arithmetic shift
    // saturate on overflow: if a,b nonzero and r==0 with wrong sign... the
    // TPE path never saturates (proven ranges); keep the exact shift only.
    return r;
}

// Q32.31 LUT-interp sin/cos, verbatim port of fixed.rs::lut_interp.
// is_sin: 1 -> sin table, 0 -> cos table. Bit-exact vs CPU (all integer ops;
// Rust % is truncated-remainder = Metal % semantics; saturating_sub only
// triggers off-path since frac_val = theta - idx*step >= 0 always on-path).
inline long q32_lut_interp(long t0, int is_sin) {
    long rem = t0 % TPE_TWO_PI;
    long theta = rem < 0 ? rem + TPE_TWO_PI : rem;
    long idx = (theta / TPE_LUT_STEP) % 256;
    long frac_val = theta - idx * TPE_LUT_STEP;
    long v1 = is_sin ? TPE_SIN_LUT[idx] : TPE_COS_LUT[idx];
    long v2 = is_sin ? TPE_SIN_LUT[(idx + 1) % 256] : TPE_COS_LUT[(idx + 1) % 256];
    long delta = v2 - v1;
    long prod = frac_val * delta;           // <= 2^53 on-path, exact
    return v1 + (prod >> 31);
}

// TRUE FUSION (dependency analysis 2026-10-27): this kernel's step uses
// ONLY threadprivate state + drive — no cross-thread reduction — so k_steps
// fold sequentially with zero barriers. Drives are STEP-MAJOR
// (df[s*n+id]) so each fused step consumes its own drive row; the old
// frozen-drive form (df[id] every step) is retired — it applied step-0's
// drive k times (caught by gate design review, never certified).

// TPE coherence kernel: GPU port of TorPhaseEncoder::evolve_vector (torus.rs:168).
// GRID: one thread per input TOKEN (batch dim); each thread folds its d/2 pairs
// sequentially (state is sequential across pairs — same dependency as the CPU loop).
// SCOPE: trajectory fields consumed by step_and_readout (theta_f/s, rapidity,
// cumulative_rotations, h3, raw_modes/basis) — full readout stays CPU.
// CERT: relative (attention-softmax-irrelevant): weights 1e-6, trajectory 1e-5.
// FoldAttention pattern (declared reference): w_rope's omega table is a closed
// form of (p, d) — computed inline via exp2/log (exact argument, one rounding);
// no iterative discovery, no provisional rescaling.
// TPE phase step inlined below is the CERTIFIED tpe_phase path (bitwise phases).

kernel void tpe_coherence_fused(
    device const float* xs [[buffer(0)]],   // [batch, d] row-major input vectors
    device float* traj_out [[buffer(1)]],   // [batch, 16]: tf,ts,rapidity,cumrot,h3[3],raw[5],sqrtbasis[4]+amp
    constant float& rope_theta [[buffer(2)]], // 10000.0 (declared reference)
    constant uint& d [[buffer(3)]],
    constant uint& batch [[buffer(4)]],
    uint id [[thread_position_in_grid]]
) {
    if (id >= batch) return;
    device const float* x = xs + (uint64_t)id * d;
    // f32 copies of the Q32 constants for the tolerance (unwrapped) path.
    // Phase path uses the TPE_* longs directly (bitwise); the f32 copies only
    // feed sin() arguments whose results are tolerance-gated.
    float omega_f = (float)TPE_OMEGA_F / 2147483648.0f;
    float omega_s = (float)TPE_OMEGA_S / 2147483648.0f;
    float k_f = (float)TPE_K_F / 2147483648.0f;
    float k_s = (float)TPE_K_S / 2147483648.0f;
    float phase_scale = (float)TPE_PHASE_SCALE / 2147483648.0f;
    // RMS over the token vector
    float nrm = 0.0f;
    for (uint i = 0u; i < d; i++) { float v = x[i]; nrm += v * v; }
    float rms = sqrt(nrm / (float)d);
    rms = rms < 1e-6f ? 1e-6f : rms;
    // Q32 TPE state (bitwise path, same as tpe_phase)
    long tf = 0l, ts = 0l;
    float amp = 0.0f;
    float unwrapped_f = 0.0f, unwrapped_s = 0.0f, unwrapped_arc = 0.0f;
    float raw[5] = {0.0f, 0.0f, 0.0f, 0.0f, 0.0f};
    float bnorm[5] = {0.0f, 0.0f, 0.0f, 0.0f, 0.0f};
    float phi3[3] = {0.70710678f, 0.0f, 0.0f};
    uint num_pairs = (d + 1u) / 2u;
    for (uint p = 0u; p < num_pairs; p++) {
        float x0 = x[2u * p];
        float x1 = (2u * p + 1u < d) ? x[2u * p + 1u] : 0.0f;
        // drive norms: CPU is Q32(x/(rms*3)) * phase_scale (Q32 mul).
        // f32 drive floats for the tolerance path; Q32 quants for the phase step.
        float df = x0 / (rms * 3.0f) * phase_scale;
        float ds = x1 / (rms * 3.0f) * phase_scale;
        // TPE phase step (fixed-point, bitwise — mirrors tpe_phase kernel).
        // CPU: df_q32 = Q32_31::from_f64(df_norm) * phase_scale (Q32 mul).
        // from_f64 rounds to nearest; rint matches it (halves measure-zero here).
        long df_q = q32_mul((long)rint((x0 / (rms * 3.0f)) * 2147483648.0f), TPE_PHASE_SCALE);
        long ds_q = q32_mul((long)rint((x1 / (rms * 3.0f)) * 2147483648.0f), TPE_PHASE_SCALE);
        long diff_sf = ts - tf;
        long diff_fs = tf - ts;
        long sin_sf = q32_lut_interp(diff_sf, 1);
        long sin_fs = q32_lut_interp(diff_fs, 1);
        long cpl_f = q32_mul(TPE_K_F, sin_sf);
        long cpl_s = q32_mul(TPE_K_S, sin_fs);
        // NOTE: omega/k/phase_scale arrive as f32 params; the CPU uses Q32
        // constants. Convert once per thread (exact: Q32 values fit f32? NO —
        // 0.1*2^31 needs 28 bits. Use nearest f32 then back: the DRIVE path is
        // tolerance-gated anyway (coherence cert is relative 1e-5).
        long step_f = TPE_OMEGA_F + cpl_f + df_q;
        long step_s = TPE_OMEGA_S + cpl_s + ds_q;
        long next_f = tf + step_f;
        long next_s = ts + step_s;
        // unwrapped track (f32, tolerance path): replicate CPU f64 formula
        float pf = (float)tf / 2147483648.0f;
        float ps = (float)ts / 2147483648.0f;
        float sdf = omega_f + k_f * sin(ps - pf) + df;
        float sds = omega_s + k_s * sin(pf - ps) + ds;
        unwrapped_f += sdf; unwrapped_s += sds;
        unwrapped_arc += sqrt(sdf * sdf + sds * sds);
        // amplitude fold (f32)
        float sff = (float)step_f / 2147483648.0f;
        float ssf = (float)step_s / 2147483648.0f;
        amp += sqrt(sff * sff + ssf * ssf) / 6.283185307f + sqrt(df * df + ds * ds);
        tf = next_f; ts = next_s;
        // shell weight (f32; radius_for_rotations: r_min+span*(0.35w+0.65 ln1p(w)))
        float rot_p = unwrapped_arc / 6.283185307f;
        float w = rot_p > 0.0f ? rot_p : 0.0f;
        float shell = 0.22f + 0.68f * (0.35f * w + 0.65f * log(1.0f + w)) + 0.05f * amp;
        // phi(psi): psi in Q32; evaluate via fixed LUT path (bitwise) then to f32
        long psi = next_f - (next_s << 1);
        float ph0 = 0.70710678f;
        float ph1 = 0.70710678f * ((float)q32_lut_interp(psi, 0) / 2147483648.0f);
        float ph2 = 0.70710678f * ((float)q32_lut_interp(psi, 1) / 2147483648.0f);
        phi3[0] = ph0; phi3[1] = ph1; phi3[2] = ph2;
        // w_rope: declared reference (FoldAttention pattern) — closed form
        float freq_exp = (float)(2u * p) / (float)(d < 2u ? 2u : d);
        float om = exp2(-freq_exp * 13.28771238f);  // 10000^-e = 2^(-e*log2(10000))
        float w_rope = 1.0f / sqrt(1.0f + (16.0f * om) * (16.0f * om));
        float w_total = w_rope * (0.92f + 0.08f * (shell / (1.0f + shell)));
        // carriers + basis MACs
        for (uint sub = 0u; sub < 2u; sub++) {
            if (2u * p + sub >= d) break;
            float kv = (sub == 0u) ? x0 : x1;
            float parity = (sub == 0u) ? 1.0f : -0.61803398875f;
            for (uint r = 0u; r < 5u; r++) {
                // Integer pre-reduction, EXACT-T variant (CERTIFIED 2026-10-27):
                // ha = ab*g + c with ab an EXACT small int. Reduce ab by the
                // golden period FIRST, then trig on the O(1) remainder.
                // Constants partition the EXACT T = 2pi_f64/g_f64 = 3.8832220775
                // (NOT f32(2pi)/g32 — that carries f32(2pi) rounding 2.4e-7,
                // x q=3296 = 8e-4-class ab error, diagnosed this pass):
                // T_hi = 3.8828125 (10 bits, q*T_hi EXACT), T_lo = f32(T-T_hi),
                // invT = f32(1/T). Quotient safety PROVEN over d<=2560
                // (0 mismatches except pathological ab=5520, caught by the
                // remainder guard). Worst cos err: 3.9e-7 @ab<=640, 4.8e-7
                // @ab<=12800 (emulation oracle). h3 rel target 1e-5 HOLDS.
                float ab = ((float)(r + 1u)) * ((float)(p + 1u));  // exact int
                float q = rint(ab * 0.2575181127f);  // 1/T_exact
                float ab_red = (ab - q * 3.8828125f) - q * 0.0004095774f;
                if (ab_red > 1.9416110516f) { q += 1.0f; ab_red -= 3.8832220775f; }
                else if (ab_red < -1.9416110516f) { q -= 1.0f; ab_red += 3.8832220775f; }
                float inner = ab_red * 1.61803398875f;
                float e_mul = fma(ab_red, 1.61803398875f, -inner);
                float ha = (inner + e_mul) + ((float)r) * 0.78539816339f;
                float ch, sh;
                sh = sincos(ha, ch);
                float carrier;
                if (r < 3u) { carrier = 0.96f * ch + 0.04f * phi3[r]; }
                else if (r == 3u) { carrier = 0.96f * ch + 0.04f * cos(unwrapped_f); }
                else { carrier = 0.96f * sh + 0.04f * sin(unwrapped_s); }
                // sin(ha*0.5): direct sincos (ha O(1) post-reduction; the
                // s2 = sh/(2*cos(ha/2)) form divides by near-zero ch2).
                // BRANCH CORRECTION (2026-10-27): ha is reduced by q*2pi, so
                // ha/2 is off by q*pi from the CPU's unreduced angle — s2
                // flips sign for odd q (measured h3 rel 1.285 with the flip
                // uncorrected). (-1)^q via q parity (q exact int in f32).
                float ch2;
                float s2 = sincos(ha * 0.5f, ch2);
                s2 *= (fmod(q, 2.0f) == 0.0f) ? 1.0f : -1.0f;
                float b = w_total * (carrier + 0.5f * parity * s2);
                raw[r] += kv * b;
                bnorm[r] += b * b;
            }
        }
    }
    // normalize + tail scalars
    for (uint r = 0u; r < 5u; r++) {
        float bn = sqrt(bnorm[r]);
        bn = bn < 1e-12f ? 1e-12f : bn;
        raw[r] /= bn;
    }
    float cumrot = unwrapped_arc / 6.283185307f;
    // rapidity_for_state: 0.22 + 0.38w + 0.28amp + 0.20 ln(1+rms).
    // Metal has no log1p; log(1+rms) with rms>=1e-6 is exact to 0.5ulp here (tolerance path).
    float rap = 0.22f + 0.38f * (cumrot > 0.0f ? cumrot : 0.0f) + 0.28f * (amp > 0.0f ? amp : 0.0f) + 0.20f * log(1.0f + rms);
// (h3n removed 2026-10-27: double-normalization bug — raw[0..2] already normalized)
    // out layout [16]: tf,ts (as float), rapidity, cumrot, h3[3], raw[5], basis_sqrt[5]... pack:
    // [0]=tf_q32 as float, [1]=ts_q32 as float, [2]=rapidity, [3]=cumrot,
    // [4..6]=h3=raw[0..2]/h3n, [7..11]=raw[0..4], [12..15]=sqrt(bnorm[0..3]) + amp
    traj_out[id * 16u + 0u] = (float)tf;
    traj_out[id * 16u + 1u] = (float)ts;
    traj_out[id * 16u + 2u] = rap;
    traj_out[id * 16u + 3u] = cumrot;
    // helical_tangent_h3 = normalized raw_modes[0..2] (CPU normalizes raw by
    // basis BEFORE forming h3 — PROVEN by probe: gpu raw[0..2] == cpu h3 to
    // 4 decimals. No second normalization (that was a double-norm bug, rel~1.0).
    traj_out[id * 16u + 4u] = raw[0];
    traj_out[id * 16u + 5u] = raw[1];
    traj_out[id * 16u + 6u] = raw[2];
    traj_out[id * 16u + 7u] = raw[0];
    traj_out[id * 16u + 8u] = raw[1];
    traj_out[id * 16u + 9u] = raw[2];
    traj_out[id * 16u + 10u] = raw[3];
    traj_out[id * 16u + 11u] = raw[4];
    traj_out[id * 16u + 12u] = sqrt(bnorm[0]);
    traj_out[id * 16u + 13u] = sqrt(bnorm[1]);
    traj_out[id * 16u + 14u] = sqrt(bnorm[2]);
    traj_out[id * 16u + 15u] = amp;
}
