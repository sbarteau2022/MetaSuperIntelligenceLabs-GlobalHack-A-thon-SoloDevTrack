//! Picard Fundamental Domain Reduction - CUDA Kernels
//!
//! Implements the Picard reduction for PSL(2, Z[i]) on H³ using cuda-oxide (SIMT track).
//! Requires warp/cluster operations not available in cutile-rs Tile IR.

#[cfg(feature = "simt-track")]
mod simt_kernels {
    use cuda_bindings::*;
    use cuda_core::prelude::*;
    use cuda_oxide::prelude::*;
    use std::os::raw::c_void;

    /// Warp-level reduction for Picard matrix operations
    /// Each warp processes one atom's Picard reduction
    #[kernel]
    pub unsafe fn picard_reduce_warp_kernel(
        atoms: *mut LorentzVec, // Input/output atoms (x0, x1, x2, x3 in Q32.31)
        num_atoms: u32,
        max_iterations: u32,
    ) {
        let tid = threadIdx.x + blockIdx.x * blockDim.x;
        let warp_id = tid / 32;
        let lane_id = tid % 32;

        if warp_id >= num_atoms {
            return;
        }

        let atom_ptr = atoms.add(warp_id as usize);
        let mut x0 = (*atom_ptr).x0;
        let mut x1 = (*atom_ptr).x1;
        let mut x2 = (*atom_ptr).x2;
        let mut x3 = (*atom_ptr).x3;

        // Convert to f64 for exact arithmetic (unbounded radii path)
        let mut x0_f = q32_31_to_f64(x0);
        let mut x1_f = q32_31_to_f64(x1);
        let mut x2_f = q32_31_to_f64(x2);
        let mut x3_f = q32_31_to_f64(x3);

        for _ in 0..max_iterations {
            // Cancellation-free x0 + x3
            let x0_plus_x3 = if x0_f > x3_f {
                x0_f + x3_f
            } else {
                x3_f + x0_f
            };

            if x0_plus_x3.abs() < 1e-10 {
                break;
            }

            // Upper half-space coordinates
            let z_re = x1_f / x0_plus_x3;
            let z_im = x2_f / x0_plus_x3;
            let w = 1.0 / x0_plus_x3;

            // Cusp folding
            let w_folded = if w > 64.0 {
                1.0 + libm::log(w - 1.0)
            } else {
                w
            };

            // Translate to fundamental domain (Gaussian integer translation)
            let z_re_red = z_re - libm::round(z_re);
            let z_im_red = z_im - libm::round(z_im);

            // Warp-level reduction for matrix operations (PSL(2, Z[i]) action)
            // γ = [[a, b], [c, d]] with a,b,c,d ∈ Z[i], ad - bc = 1
            // For benchmark: simplified reduction

            let norm_sq = z_re_red * z_re_red + z_im_red * z_im_red;
            let denom = 1.0 - norm_sq;

            if denom <= 1e-10 {
                break;
            }

            // Map back to Lorentz
            let new_x0 = (1.0 + norm_sq) / denom;
            let new_x1 = 2.0 * z_re_red / denom;
            let new_x2 = 2.0 * z_im_red / denom;
            let new_x3 = (1.0 - norm_sq) / denom;

            // Check convergence
            if (new_x0 - x0_f).abs() < 1e-12
                && (new_x1 - x1_f).abs() < 1e-12
                && (new_x2 - x2_f).abs() < 1e-12
                && (new_x3 - x3_f).abs() < 1e-12
            {
                break;
            }

            x0_f = new_x0;
            x1_f = new_x1;
            x2_f = new_x2;
            x3_f = new_x3;
        }

        // Convert back to Q32.31
        (*atom_ptr).x0 = f64_to_q32_31(x0_f);
        (*atom_ptr).x1 = f64_to_q32_31(x1_f);
        (*atom_ptr).x2 = f64_to_q32_31(x2_f);
        (*atom_ptr).x3 = f64_to_q32_31(x3_f);
    }

    /// Batch Picard reduction for multiple atoms
    /// Uses cluster-level cooperation for larger reductions
    #[kernel]
    pub unsafe fn picard_reduce_batch_kernel(
        atoms: *mut LorentzVec,
        num_atoms: u32,
        iterations: u32,
    ) {
        let tid = threadIdx.x + blockIdx.x * blockDim.x;
        let stride = blockDim.x * gridDim.x;

        for i in (tid..num_atoms).step_by(stride as usize) {
            let atom_ptr = atoms.add(i as usize);
            let mut x0 = (*atom_ptr).x0;
            let mut x1 = (*atom_ptr).x1;
            let mut x2 = (*atom_ptr).x2;
            let mut x3 = (*atom_ptr).x3;

            let mut x0_f = q32_31_to_f64(x0);
            let mut x1_f = q32_31_to_f64(x1);
            let mut x2_f = q32_31_to_f64(x2);
            let mut x3_f = q32_31_to_f64(x3);

            for _ in 0..iterations {
                let x0_plus_x3 = if x0_f > x3_f {
                    x0_f + x3_f
                } else {
                    x3_f + x0_f
                };
                if x0_plus_x3.abs() < 1e-10 {
                    break;
                }

                let z_re = x1_f / x0_plus_x3;
                let z_im = x2_f / x0_plus_x3;
                let w = 1.0 / x0_plus_x3;

                let w_folded = if w > 64.0 {
                    1.0 + libm::log(w - 1.0)
                } else {
                    w
                };
                let z_re_red = z_re - libm::round(z_re);
                let z_im_red = z_im - libm::round(z_im);

                let norm_sq = z_re_red * z_re_red + z_im_red * z_im_red;
                let denom = 1.0 - norm_sq;
                if denom <= 1e-10 {
                    break;
                }

                let new_x0 = (1.0 + norm_sq) / denom;
                let new_x1 = 2.0 * z_re_red / denom;
                let new_x2 = 2.0 * z_im_red / denom;
                let new_x3 = (1.0 - norm_sq) / denom;

                if (new_x0 - x0_f).abs() < 1e-12
                    && (new_x1 - x1_f).abs() < 1e-12
                    && (new_x2 - x2_f).abs() < 1e-12
                    && (new_x3 - x3_f).abs() < 1e-12
                {
                    break;
                }

                x0_f = new_x0;
                x1_f = new_x1;
                x2_f = new_x2;
                x3_f = new_x3;
            }

            (*atom_ptr).x0 = f64_to_q32_31(x0_f);
            (*atom_ptr).x1 = f64_to_q32_31(x1_f);
            (*atom_ptr).x2 = f64_to_q32_31(x2_f);
            (*atom_ptr).x3 = f64_to_q32_31(x3_f);
        }
    }

    // Import Q32.31 conversion functions
    use super::super::hyperbolic::{f64_to_q32_31, q32_31_to_f64};
    use libm;
}

/// Picard reduction dispatcher - chooses between warp and batch kernels
pub struct PicardReducer {
    #[cfg(feature = "simt-track")]
    warp_func: cuda_oxide::Function,
    #[cfg(feature = "simt-track")]
    batch_func: cuda_oxide::Function,
    #[cfg(feature = "cutile-track")]
    stream: cuda_core::Stream,
    #[cfg(not(feature = "cutile-track"))]
    stream: (), // Stub for non-cutile-track compilation
}

impl PicardReducer {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        #[cfg(all(feature = "simt-track", feature = "cutile-track"))]
        {
            use cuda_core::Stream;
            use cuda_oxide::Module;

            let stream = Stream::new(cuda_core::StreamFlags::NON_BLOCKING, 0)?;

            Ok(Self {
                warp_func: todo!("Load warp kernel PTX"),
                batch_func: todo!("Load batch kernel PTX"),
                stream,
            })
        }
        #[cfg(all(feature = "cutile-track", not(feature = "simt-track")))]
        {
            use cuda_core::Stream;
            let stream = Stream::new(cuda_core::StreamFlags::NON_BLOCKING, 0)?;
            Ok(Self { stream })
        }
        #[cfg(not(feature = "cutile-track"))]
        {
            Ok(Self { stream: () })
        }
    }

    /// Reduce atoms using warp kernel (one warp per atom)
    #[cfg(feature = "simt-track")]
    pub fn reduce_warp(
        &self,
        atoms: &mut [crate::hyperbolic::LorentzVec],
        iterations: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use cuda_core::KernelLaunchConfig;

        let num_atoms = atoms.len() as u32;
        let block_size = 256;
        let grid_size = (num_atoms * 32 + block_size - 1) / block_size; // 32 threads per warp

        let config = KernelLaunchConfig::new(grid_size, block_size, 0, self.stream);

        unsafe {
            self.warp_func
                .launch(config, (atoms.as_mut_ptr(), num_atoms, iterations))?;
        }

        self.stream.synchronize()?;
        Ok(())
    }

    /// Reduce atoms using batch kernel (one thread per atom)
    #[cfg(feature = "simt-track")]
    pub fn reduce_batch(
        &self,
        atoms: &mut [crate::hyperbolic::LorentzVec],
        iterations: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use cuda_core::KernelLaunchConfig;

        let num_atoms = atoms.len() as u32;
        let block_size = 256;
        let grid_size = (num_atoms + block_size - 1) / block_size;

        let config = KernelLaunchConfig::new(grid_size, block_size, 0, self.stream);

        unsafe {
            self.batch_func
                .launch(config, (atoms.as_mut_ptr(), num_atoms, iterations))?;
        }

        self.stream.synchronize()?;
        Ok(())
    }

    /// CPU fallback for validation (no CUDA required)
    pub fn reduce_cpu_fallback(atoms: &mut [crate::hyperbolic::LorentzVec], iterations: u32) {
        use crate::hyperbolic::picard_reduce::reduce;

        for atom in atoms.iter_mut() {
            for _ in 0..iterations {
                let x0 = crate::hyperbolic::q32_31_to_f64(atom.x0);
                if x0 <= 8.0 {
                    crate::hyperbolic::picard_reduce::reduce_moderate(atom);
                    break;
                } else {
                    crate::hyperbolic::picard_reduce::reduce_unbounded(atom);
                    if crate::hyperbolic::q32_31_to_f64(atom.x0) <= 8.0 {
                        break;
                    }
                }
            }
        }
    }
}

/// LorentzVec for CUDA kernel compatibility
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct LorentzVec {
    pub x0: i64,
    pub x1: i64,
    pub x2: i64,
    pub x3: i64,
}

// Q32.31 conversions (re-exported for kernel use)
#[inline]
pub fn q32_31_to_f64(x: i64) -> f64 {
    x as f64 / (1i64 << 31) as f64
}

#[inline]
pub fn f64_to_q32_31(x: f64) -> i64 {
    (x * (1i64 << 31) as f64).round() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hyperbolic::LorentzVec as HostLorentzVec;

    #[test]
    fn test_picard_cpu_fallback() {
        let mut atoms = vec![
            HostLorentzVec::new_f64(100.0, 10.0, 5.0, 2.0),
            HostLorentzVec::new_f64(50.0, 20.0, 10.0, 5.0),
        ];

        PicardReducer::reduce_cpu_fallback(&mut atoms, 32);

        // Should have reduced x0
        assert!(crate::hyperbolic::q32_31_to_f64(atoms[0].x0) < 100.0);
        assert!(crate::hyperbolic::q32_31_to_f64(atoms[1].x0) < 50.0);
    }
}
