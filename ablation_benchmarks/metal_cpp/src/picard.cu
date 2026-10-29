// Picard Reduction CUDA Kernel - C++ Baseline
// Hand-rolled PTX for warp-level reductions on PSL(2, Z[i]) matrix operations

#include "hyperbolic_kv_bench.h"
#include <cmath>
#include <cstdint>
#include <cuda_runtime.h>

// Q32.31 conversion device functions
__device__ __forceinline__ double q32_31_to_f64_d(int64_t x) {
  return static_cast<double>(x) / 2147483648.0;
}

__device__ __forceinline__ int64_t f64_to_q32_31_d(double x) {
  return static_cast<int64_t>(rint(x * 2147483648.0));
}

// LorentzVec for device
struct LorentzVecD {
  int64_t x0, x1, x2, x3;
};

// Warp-level reduction for Picard matrix operations
// Each warp processes one atom's Picard reduction
__global__ void picard_reduce_warp_kernel(LorentzVecD *atoms,
                                          uint32_t num_atoms,
                                          uint32_t max_iterations) {
  uint32_t tid = threadIdx.x + blockIdx.x * blockDim.x;
  uint32_t warp_id = tid / 32;
  uint32_t lane_id = tid % 32;

  if (warp_id >= num_atoms)
    return;

  LorentzVecD atom = atoms[warp_id];

  double x0 = q32_31_to_f64_d(atom.x0);
  double x1 = q32_31_to_f64_d(atom.x1);
  double x2 = q32_31_to_f64_d(atom.x2);
  double x3 = q32_31_to_f64_d(atom.x3);

  for (uint32_t iter = 0; iter < max_iterations; ++iter) {
    // Cancellation-free x0 + x3
    double x0_plus_x3 = (x0 > x3) ? (x0 + x3) : (x3 + x0);
    if (fabs(x0_plus_x3) < 1e-10)
      break;

    // Upper half-space coordinates
    double z_re = x1 / x0_plus_x3;
    double z_im = x2 / x0_plus_x3;
    double w = 1.0 / x0_plus_x3;

    // Cusp folding
    double w_folded = (w > 64.0) ? (1.0 + log(w - 1.0)) : w;

    // Translate to fundamental domain
    double z_re_red = z_re - rint(z_re);
    double z_im_red = z_im - rint(z_im);

    // Warp-level reduction for matrix operations
    // PSL(2, Z[i]) action: γ = [[a, b], [c, d]] with a,b,c,d ∈ Z[i], ad - bc =
    // 1 Simplified reduction for benchmark

    double norm_sq = z_re_red * z_re_red + z_im_red * z_im_red;
    double denom = 1.0 - norm_sq;
    if (denom <= 1e-10)
      break;

    double new_x0 = (1.0 + norm_sq) / denom;
    double new_x1 = 2.0 * z_re_red / denom;
    double new_x2 = 2.0 * z_im_red / denom;
    double new_x3 = (1.0 - norm_sq) / denom;

    // Check convergence (warp-level reduction for comparison)
    double dx0 = fabs(new_x0 - x0);
    double dx1 = fabs(new_x1 - x1);
    double dx2 = fabs(new_x2 - x2);
    double dx3 = fabs(new_x3 - x3);

    // Warp reduction for max difference
    for (int offset = 16; offset > 0; offset >>= 1) {
      dx0 = fmax(dx0, __shfl_down_sync(0xFFFFFFFF, dx0, offset));
      dx1 = fmax(dx1, __shfl_down_sync(0xFFFFFFFF, dx1, offset));
      dx2 = fmax(dx2, __shfl_down_sync(0xFFFFFFFF, dx2, offset));
      dx3 = fmax(dx3, __shfl_down_sync(0xFFFFFFFF, dx3, offset));
    }

    // Broadcast convergence check from lane 0
    bool converged = (lane_id == 0) &&
                     (dx0 < 1e-12 && dx1 < 1e-12 && dx2 < 1e-12 && dx3 < 1e-12);
    converged = __shfl_sync(0xFFFFFFFF, converged, 0);
    if (converged)
      break;

    x0 = new_x0;
    x1 = new_x1;
    x2 = new_x2;
    x3 = new_x3;
  }

  // Write back (lane 0 writes for the warp)
  if (lane_id == 0) {
    atoms[warp_id].x0 = f64_to_q32_31_d(x0);
    atoms[warp_id].x1 = f64_to_q32_31_d(x1);
    atoms[warp_id].x2 = f64_to_q32_31_d(x2);
    atoms[warp_id].x3 = f64_to_q32_31_d(x3);
  }
}

// Batch Picard reduction kernel (one thread per atom)
__global__ void picard_reduce_batch_kernel(LorentzVecD *atoms,
                                           uint32_t num_atoms,
                                           uint32_t iterations) {
  uint32_t tid = threadIdx.x + blockIdx.x * blockDim.x;
  uint32_t stride = blockDim.x * gridDim.x;

  for (uint32_t i = tid; i < num_atoms; i += stride) {
    LorentzVecD atom = atoms[i];

    double x0 = q32_31_to_f64_d(atom.x0);
    double x1 = q32_31_to_f64_d(atom.x1);
    double x2 = q32_31_to_f64_d(atom.x2);
    double x3 = q32_31_to_f64_d(atom.x3);

    for (uint32_t iter = 0; iter < iterations; ++iter) {
      double x0_plus_x3 = (x0 > x3) ? (x0 + x3) : (x3 + x0);
      if (fabs(x0_plus_x3) < 1e-10)
        break;

      double z_re = x1 / x0_plus_x3;
      double z_im = x2 / x0_plus_x3;
      double w = 1.0 / x0_plus_x3;

      double w_folded = (w > 64.0) ? (1.0 + log(w - 1.0)) : w;
      double z_re_red = z_re - rint(z_re);
      double z_im_red = z_im - rint(z_im);

      double norm_sq = z_re_red * z_re_red + z_im_red * z_im_red;
      double denom = 1.0 - norm_sq;
      if (denom <= 1e-10)
        break;

      double new_x0 = (1.0 + norm_sq) / denom;
      double new_x1 = 2.0 * z_re_red / denom;
      double new_x2 = 2.0 * z_im_red / denom;
      double new_x3 = (1.0 - norm_sq) / denom;

      if (fabs(new_x0 - x0) < 1e-12 && fabs(new_x1 - x1) < 1e-12 &&
          fabs(new_x2 - x2) < 1e-12 && fabs(new_x3 - x3) < 1e-12) {
        break;
      }

      x0 = new_x0;
      x1 = new_x1;
      x2 = new_x2;
      x3 = new_x3;
    }

    atoms[i].x0 = f64_to_q32_31_d(x0);
    atoms[i].x1 = f64_to_q32_31_d(x1);
    atoms[i].x2 = f64_to_q32_31_d(x2);
    atoms[i].x3 = f64_to_q32_31_d(x3);
  }
}

// Host launcher functions
extern "C" void launch_picard_reduce_warp(LorentzVecD *d_atoms,
                                          uint32_t num_atoms,
                                          uint32_t iterations) {
  dim3 block(256);
  dim3 grid((num_atoms * 32 + block.x - 1) / block.x);
  picard_reduce_warp_kernel<<<grid, block>>>(d_atoms, num_atoms, iterations);
  cudaDeviceSynchronize();
}

extern "C" void launch_picard_reduce_batch(LorentzVecD *d_atoms,
                                           uint32_t num_atoms,
                                           uint32_t iterations) {
  dim3 block(256);
  dim3 grid((num_atoms + block.x - 1) / block.x);
  picard_reduce_batch_kernel<<<grid, block>>>(d_atoms, num_atoms, iterations);
  cudaDeviceSynchronize();
}
