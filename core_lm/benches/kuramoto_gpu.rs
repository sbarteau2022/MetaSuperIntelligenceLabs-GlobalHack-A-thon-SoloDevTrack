//! GPU bench: kuramoto_meanfield_fused wall time vs CPU mean-field + all-pairs.
//! Run: cargo bench --manifest-path core_lm/Cargo.toml --bench kuramoto_gpu
use core_lm::geometry::torus::kuramoto_resultant_step;
use core_lm::lock::kuramoto_step_cpu;
use std::time::Instant;

fn det_vec(n: usize, seed: u64) -> Vec<i64> {
    let mut x = seed;
    (0..n)
        .map(|_| {
            x = x
                .wrapping_mul(0x9E3779B97F4A7C15)
                .wrapping_add(0xBF58476D1CE4E5B9);
            ((x >> 33) as i32) as i64
        })
        .collect()
}

fn main() {
    #[cfg(not(target_os = "macos"))]
    {
        println!("skipped: Metal bench requires macOS");
        return;
    }
    #[cfg(target_os = "macos")]
    {
        use metal::*;
        let device = Device::system_default().expect("no Metal device");
        let msl = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/kuramoto_meanfield.metal"
        ))
        .unwrap();
        let lib = device
            .new_library_with_source(&msl, &CompileOptions::new())
            .unwrap();
        let func = lib.get_function("kuramoto_meanfield_fused", None).unwrap();
        let pipe = device
            .new_compute_pipeline_state_with_function(&func)
            .unwrap();
        println!("n, gpu_us_per_step, cpu_meanfield_us, cpu_allpairs_us, gpu_speedup_vs_allpairs");
        for &n in &[16usize, 64, 256, 1024, 2560] {
            let theta0 = det_vec(n, 0x1234);
            let omega = det_vec(n, 0x5678);
            let dt: i64 = (0.05 * ((1i64 << 31) as f64)) as i64;
            let k: i64 = (0.3 * ((1i64 << 31) as f64)) as i64;
            // CPU baselines (release).
            let mut x = theta0.clone();
            let t = Instant::now();
            for _ in 0..20 {
                let mut c = x.clone();
                kuramoto_resultant_step(&mut c, &omega, dt, k);
                std::hint::black_box(&c);
            }
            let mf = t.elapsed().as_micros() as f64 / 20.0;
            let t = Instant::now();
            let reps = if n > 256 { 3 } else { 20 };
            for _ in 0..reps {
                let mut c = x.clone();
                kuramoto_step_cpu(&mut c, &omega, dt, k);
                std::hint::black_box(&c);
            }
            let ap = t.elapsed().as_micros() as f64 / reps as f64;
            x = theta0.clone();
            let _ = &x;
            // GPU: persistent buffers, k_steps=1, timed over 200 dispatches.
            let queue = device.new_command_queue();
            let mk = |v: &[i64]| {
                device.new_buffer_with_data(
                    v.as_ptr() as *const _,
                    (v.len() * 8) as u64,
                    MTLResourceOptions::StorageModeShared,
                )
            };
            let tb = mk(&theta0);
            let ob = mk(&omega);
            let dout = vec![0i64; n];
            let db = mk(&dout);
            let n_u32 = n as u32;
            let one: u32 = 1;
            // warmup
            for _ in 0..5 {
                let cb = queue.new_command_buffer();
                let enc = cb.new_compute_command_encoder();
                enc.set_compute_pipeline_state(&pipe);
                enc.set_buffer(0, Some(&tb), 0);
                enc.set_buffer(1, Some(&ob), 0);
                enc.set_buffer(2, Some(&db), 0);
                enc.set_bytes(3, 8, &dt as *const _ as *const _);
                enc.set_bytes(4, 8, &k as *const _ as *const _);
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
            }
            let t = Instant::now();
            let iters = 200u32;
            for _ in 0..iters {
                let cb = queue.new_command_buffer();
                let enc = cb.new_compute_command_encoder();
                enc.set_compute_pipeline_state(&pipe);
                enc.set_buffer(0, Some(&tb), 0);
                enc.set_buffer(1, Some(&ob), 0);
                enc.set_buffer(2, Some(&db), 0);
                enc.set_bytes(3, 8, &dt as *const _ as *const _);
                enc.set_bytes(4, 8, &k as *const _ as *const _);
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
            }
            let gpu = t.elapsed().as_micros() as f64 / iters as f64;
            println!(
                "{},{:.1},{:.1},{:.1},{:.1}x",
                n,
                gpu,
                mf,
                ap,
                ap / gpu.max(0.05)
            );
        }
    }
}
