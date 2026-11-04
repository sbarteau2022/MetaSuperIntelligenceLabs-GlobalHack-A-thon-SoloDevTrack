// q32_cordic.metal — integer-only CORDIC sin/cos + Q32 multiply.
// Ported VERBATIM from core_lm/src/lock.rs (cordic_sin, CORDIC_GAIN, ATAN_TABLE).
// NO FLOAT ANYWHERE IN THIS FILE. All angles pi-unit Q32.31 (1<<31 = pi rad),
// all outputs Q32.31 integers. cos via sin(x + pi/2): exact, no second table.

constant long CORDIC_GAIN = 1304065748;   // 0.6072529350088814 * 2^31 (lock.rs, corrected)
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

// Bit-exact port of lock.rs::cordic_sin. theta in pi-units.
// Wrapping: Metal long arithmetic wraps mod 2^64 like Rust wrapping_*.
// >> on negative long is arithmetic, matching Rust >>.
inline long q32_cordic_sin(long theta) {
    long z = (long)(int)theta;              // hardware torus wrap
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

// cos(t) = sin(t + pi/2), exact in pi-units (addition, no rounding).
inline long q32_cordic_cos(long theta) {
    return q32_cordic_sin(theta + (1l << 30));
}

// Q32.31 multiply == Rust (a as i128 * b as i128) >> 31 (FLOOR, arithmetic shift).
// VERIFIED 2026-10-27: naive split (p3<<1)+(mid>>15)+(p0>>31) is 1 LSB low on
// ~1/3 of inputs (separate floors drop the low-bit carry: 34482/100000 fails).
// FIX: fold mid+p0 BEFORE shifting: t=(mid<<16)+p0 (fits i64: mid<=~2^32 on
// certified paths => t<=~2^48), result=(p3<<1)+(t>>31), single floor carries all.
// Re-verified 0/100000 fails vs i128 truth. (Saturation: certified path products
// fit i64; out-of-range callers must clamp — matches Rust .clamp usage.)
inline long q32_mul(long a, long b) {
    long a_hi = a >> 16;
    long a_lo = a - (a_hi << 16);
    long b_hi = b >> 16;
    long b_lo = b - (b_hi << 16);
    long p0 = a_lo * b_lo;   // <= 2^32, exact
    long p1 = a_hi * b_lo;   // certified ranges: fits (proven per call site)
    long p2 = a_lo * b_hi;
    long p3 = a_hi * b_hi;
    long mid = p1 + p2;
    long t = (mid << 16) + p0;   // carry-inclusive middle+low (the fix)
    return (p3 << 1) + (t >> 31);
}
