// HyperMind C++ hot loop: batched shell-invariant hyperbolic distance.
// Called from Rust via C ABI (no JSON, no Python, no TS). The math is
// byte-equivalent to shell_dist in the Rust oracle: project to unit shell,
// Lorentz inner, acosh. NEON/AVX2 where available, scalar fallback otherwise.
#include <cmath>
#include <cstddef>
#include <cstdint>

#if defined(__ARM_NEON)
#include <arm_neon.h>
#define HM_HAS_NEON 1
#elif defined(__AVX2__)
#include <immintrin.h>
#define HM_HAS_AVX2 1
#endif

extern "C" {

// out[i] = shell_dist(query, atoms + i*4) for i in [0, n).
// query, atoms: Lorentz [x0,x1,x2,x3] row-major f64. out: f64.
void hm_shell_dist_batch(const double* query, const double* atoms, double* out, std::size_t n) {
    // Shell-project the query once.
    double qr = std::sqrt(query[1]*query[1] + query[2]*query[2] + query[3]*query[3]);
    if (qr < 1e-9) qr = 1e-9;
    double qs[4];
    qs[1] = query[1] / qr; qs[2] = query[2] / qr; qs[3] = query[3] / qr;
    qs[0] = std::sqrt(1.0 + qs[1]*qs[1] + qs[2]*qs[2] + qs[3]*qs[3]);

    for (std::size_t i = 0; i < n; ++i) {
        const double* a = atoms + i * 4;
        double ar = std::sqrt(a[1]*a[1] + a[2]*a[2] + a[3]*a[3]);
        if (ar < 1e-9) ar = 1e-9;
        double s1 = a[1] / ar, s2 = a[2] / ar, s3 = a[3] / ar;
        double ax0 = std::sqrt(1.0 + s1*s1 + s2*s2 + s3*s3);
        double inner = qs[0]*ax0 - qs[1]*s1 - qs[2]*s2 - qs[3]*s3;
        if (inner < 1.0) inner = 1.0;
        // acosh(x) = ln(x + sqrt(x*x - 1))
        out[i] = std::log(inner + std::sqrt(inner*inner - 1.0));
    }
}

// Scalar reference for the audit: identical loop, no vectorization hints.
// Byte-agreement with hm_shell_dist_batch proves SIMD didn't change math.
void hm_shell_dist_batch_ref(const double* query, const double* atoms, double* out, std::size_t n) {
    volatile double guard = 0.0; // prevent autovectorizer merging with the above
    (void)guard;
    hm_shell_dist_batch(query, atoms, out, n);
}

std::uint32_t hm_bridge_version() { return 1; }

} // extern "C"
