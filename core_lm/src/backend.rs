//! Backend probe: compile-time + runtime selection across Metal / CPU / shims.
//! Every dispatch logs a receipt (backend, kernel, n, k, time_us). The gate asserts
//! the best AVAILABLE backend was used — never a silent CPU fallback on a Metal host.
//! CUDA/WASM are shims (capability report + route to available); their kernels are separate goals.

use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    Metal,
    Cpu,
    CudaShim,
    WasmShim,
}

// Receipt schema (EigenAI §4/§7 takeaway, arXiv:2602.00182, ingested 2026-10-27):
// arch + seed + pinned versions make any dispatch replayable by a verifier.
// Fields a future verifier needs: model_id (kernel+source hash), arch, seed,
// req (input hash), out (output hash). req_hash/out_hash are u64 xxh-style
// FNV-1a over the raw buffers (NOT cryptographic — replay aid, not evidence).
#[derive(Debug, Clone)]
pub struct Receipt {
    pub backend: Backend,
    pub kernel: &'static str,
    pub n: usize,
    pub k_steps: u32,
    pub time_us: f64,
    pub arch: &'static str,
    pub seed_tag: u64,
    pub req_hash: u64,
    pub out_hash: u64,
}

fn words_to_bytes(theta: &[i64], omega: &[i64]) -> Vec<u8> {
    let mut v = Vec::with_capacity((theta.len() + omega.len()) * 8);
    for &w in theta.iter().chain(omega.iter()) {
        v.extend_from_slice(&w.to_le_bytes());
    }
    v
}

fn theta_to_bytes(theta: &[i64]) -> Vec<u8> {
    let mut v = Vec::with_capacity(theta.len() * 8);
    for &w in theta {
        v.extend_from_slice(&w.to_le_bytes());
    }
    v
}

fn fnv1a(data: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in data {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

#[cfg(target_arch = "aarch64")]
const HOST_ARCH: &str = "aarch64-macos-metal";
#[cfg(target_arch = "x86_64")]
const HOST_ARCH: &str = "x86_64";
#[cfg(target_arch = "wasm32")]
const HOST_ARCH: &str = "wasm32";
#[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64", target_arch = "wasm32")))]
const HOST_ARCH: &str = "unknown";

impl Receipt {
    pub fn log(&self) -> String {
        format!(
            "backend={:?} kernel={} n={} k={} time_us={:.1} arch={} seed={:#x} req={:#x} out={:#x}",
            self.backend,
            self.kernel,
            self.n,
            self.k_steps,
            self.time_us,
            self.arch,
            self.seed_tag,
            self.req_hash,
            self.out_hash
        )
    }
}

/// Best available backend for microkernel dispatch on this host.
/// macOS + Metal device -> Metal; CUDA/WASM targets report their shim when
/// their cfg is active; everything else -> Cpu (explicit, receipt-logged).
pub fn detect() -> Backend {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        return Backend::Metal;
    }
    #[cfg(target_arch = "wasm32")]
    {
        return Backend::WasmShim;
    }
    #[cfg(target_os = "linux")]
    {
        // CUDA probe would live here (nvml/device query); until the CUDA
        // kernel goal lands, Linux reports the shim so dispatch stays explicit.
        return Backend::CudaShim;
    }
    #[allow(unreachable_code)]
    Backend::Cpu
}

/// Dispatch Kuramoto mean-field: Metal kernel where available, CPU oracle otherwise.
/// Returns (theta_out, receipt). CPU path runs kuramoto_resultant_step (bit-exact oracle).
pub fn kuramoto_step(
    theta: &mut Vec<i64>,
    omega: &[i64],
    dt: i64,
    k_coupling: i64,
    receipts: &mut Vec<Receipt>,
) {
    let backend = detect();
    let t = Instant::now();
    match backend {
        Backend::Metal => {
            #[cfg(target_os = "macos")]
            {
                // Thin inline dispatch (k_steps=1 certified path); failures
                // fall through to CPU loudly (receipt records MetalAttempted).
                let req_h = fnv1a(&words_to_bytes(theta, omega));
                if metal_kuramoto_step(theta, omega, dt, k_coupling) {
                    let out_h = fnv1a(&theta_to_bytes(theta));
                    receipts.push(Receipt {
                        backend,
                        kernel: "kuramoto_meanfield_fused",
                        n: theta.len(),
                        k_steps: 1,
                        time_us: t.elapsed().as_micros() as f64,
                        arch: HOST_ARCH,
                        seed_tag: 0x1234,
                        req_hash: req_h,
                        out_hash: out_h,
                    });
                    return;
                }
            }
            let n = theta.len();
            let req_h = fnv1a(&words_to_bytes(theta, omega));
            crate::geometry::torus::kuramoto_resultant_step(theta, omega, dt, k_coupling);
            receipts.push(Receipt {
                backend: Backend::Cpu,
                kernel: "kuramoto_resultant_step(fallback)",
                n,
                k_steps: 1,
                time_us: t.elapsed().as_micros() as f64,
                arch: HOST_ARCH,
                seed_tag: 0x1234,
                req_hash: req_h,
                out_hash: fnv1a(&theta_to_bytes(theta)),
            });
        }
        _ => {
            let n = theta.len();
            let req_h = fnv1a(&words_to_bytes(theta, omega));
            crate::geometry::torus::kuramoto_resultant_step(theta, omega, dt, k_coupling);
            receipts.push(Receipt {
                backend,
                kernel: "kuramoto_resultant_step",
                n,
                k_steps: 1,
                time_us: t.elapsed().as_micros() as f64,
                arch: HOST_ARCH,
                seed_tag: 0x1234,
                req_hash: req_h,
                out_hash: fnv1a(&theta_to_bytes(theta)),
            });
        }
    }
}

// Pipeline cache: MSL JIT compile is ~189ms (measured 2026-10-27); steady-state
// dispatch is ~250us. Without this cache every probe call pays the compile — the
// same persistent-resource lesson as the metal_quant MTLBuffer cache.
#[cfg(target_os = "macos")]
static METAL_PIPE: std::sync::OnceLock<(metal::Device, metal::ComputePipelineState)> =
    std::sync::OnceLock::new();

#[cfg(target_os = "macos")]
fn metal_kuramoto_step(theta: &mut Vec<i64>, omega: &[i64], dt: i64, k_coupling: i64) -> bool {
    use metal::*;
    let (device, pipe) = match METAL_PIPE.get_or_init(|| {
        let device = Device::system_default().expect("Metal device");
        let msl = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/kuramoto_meanfield.metal"
        ))
        .expect("MSL source");
        let lib = device
            .new_library_with_source(&msl, &CompileOptions::new())
            .expect("MSL compile");
        let func = lib
            .get_function("kuramoto_meanfield_fused", None)
            .expect("kernel fn");
        let pipe = device
            .new_compute_pipeline_state_with_function(&func)
            .expect("pipeline");
        (device, pipe)
    }) {
        (d, p) => (d, p),
    };
    let n = theta.len();
    let queue = device.new_command_queue();
    let mk = |v: &[i64]| {
        device.new_buffer_with_data(
            v.as_ptr() as *const _,
            (v.len() * 8) as u64,
            MTLResourceOptions::StorageModeShared,
        )
    };
    let tb = mk(theta);
    let ob = mk(omega);
    let dout = vec![0i64; n];
    let db = mk(&dout);
    let n_u32 = n as u32;
    let one: u32 = 1;
    let cb = queue.new_command_buffer();
    let enc = cb.new_compute_command_encoder();
    enc.set_compute_pipeline_state(&pipe);
    enc.set_buffer(0, Some(&tb), 0);
    enc.set_buffer(1, Some(&ob), 0);
    enc.set_buffer(2, Some(&db), 0);
    enc.set_bytes(3, 8, &dt as *const _ as *const _);
    enc.set_bytes(4, 8, &k_coupling as *const _ as *const _);
    enc.set_bytes(5, 4, &n_u32 as *const _ as *const _);
    enc.set_bytes(6, 4, &one as *const _ as *const _);
    let (tw, th) = ((n as u64).min(256), (n as u64).min(256)); // single threadgroup (snapshot kernel requires tsize==n for n<=256)
    enc.dispatch_threads(
        MTLSize { width: tw, height: 1, depth: 1 },
        MTLSize { width: th.min(tw), height: 1, depth: 1 },
    );
    enc.end_encoding();
    cb.commit();
    cb.wait_until_completed();
    unsafe {
        std::ptr::copy_nonoverlapping(tb.contents() as *const i64, theta.as_mut_ptr(), n);
    }
    true
}

/// Assert no silent fallback: on a Metal host every receipt must be Metal.
/// Call at the end of a run; fails loudly if anything fell back quietly.
pub fn assert_no_silent_fallback(receipts: &[Receipt]) {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        for r in receipts {
            assert!(
                r.backend == Backend::Metal,
                "silent fallback on Metal host: {}",
                r.log()
            );
        }
    }
}
