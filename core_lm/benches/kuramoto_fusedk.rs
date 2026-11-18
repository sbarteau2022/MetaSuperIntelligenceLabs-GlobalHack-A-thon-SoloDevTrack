//! GPU bench: v2 single fused launch (k=8) vs k sequential v1 launches.
//! Run: cargo bench --manifest-path core_lm/Cargo.toml --bench kuramoto_fusedk
use std::time::Instant;
fn det_vec(n: usize, seed: u64) -> Vec<i64> {
    let mut x = seed;
    (0..n).map(|_| { x = x.wrapping_mul(0x9E3779B97F4A7C15).wrapping_add(0xBF58476D1CE4E5B9); ((x >> 33) as i32) as i64 }).collect()
}
fn main() {
    #[cfg(not(target_os = "macos"))] { println!("skipped: macOS only"); return; }
    #[cfg(target_os = "macos")] {
        use metal::*;
        let device = Device::system_default().expect("no Metal device");
        let m1 = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/kuramoto_meanfield.metal")).unwrap();
        let m2 = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/kuramoto_fused_tg.metal")).unwrap();
        let l1 = device.new_library_with_source(&m1, &CompileOptions::new()).unwrap();
        let l2 = device.new_library_with_source(&m2, &CompileOptions::new()).unwrap();
        let p1 = device.new_compute_pipeline_state_with_function(&l1.get_function("kuramoto_meanfield_fused", None).unwrap()).unwrap();
        let p2 = device.new_compute_pipeline_state_with_function(&l2.get_function("kuramoto_fused_tg", None).unwrap()).unwrap();
        println!("n, k, fused_us, sequential_us, amortization");
        for &n in &[16usize, 64, 256] {
            let k = 8u32;
            let theta0 = det_vec(n, 0x1234); let omega = det_vec(n, 0x5678);
            let dt: i64 = (0.05 * ((1i64<<31) as f64)) as i64;
            let kk: i64 = (0.3 * ((1i64<<31) as f64)) as i64;
            let queue = device.new_command_queue();
            let mk = |v: &[i64]| device.new_buffer_with_data(v.as_ptr() as *const _, (v.len()*8) as u64, MTLResourceOptions::StorageModeShared);
            let tb = mk(&theta0); let ob = mk(&omega);
            let dout = vec![0i64; n]; let db = mk(&dout);
            let n_u32 = n as u32; let one: u32 = 1;
            let go = |pipe: &ComputePipelineState, kk_: u32| {
                let cb = queue.new_command_buffer();
                let enc = cb.new_compute_command_encoder();
                enc.set_compute_pipeline_state(pipe);
                enc.set_buffer(0, Some(&tb), 0);
                enc.set_buffer(1, Some(&ob), 0);
                enc.set_buffer(2, Some(&db), 0);
                enc.set_bytes(3, 8, &dt as *const _ as *const _);
                enc.set_bytes(4, 8, &kk as *const _ as *const _);
                enc.set_bytes(5, 4, &n_u32 as *const _ as *const _);
                enc.set_bytes(6, 4, &kk_ as *const _ as *const _);
                enc.dispatch_threads(MTLSize{width:n as u64,height:1,depth:1}, MTLSize{width:n as u64,height:1,depth:1});
                enc.end_encoding(); cb.commit(); cb.wait_until_completed();
            };
            for _ in 0..5 { go(&p2, k); }
            let t = Instant::now(); for _ in 0..50 { go(&p2, k); }
            let fused = t.elapsed().as_micros() as f64 / 50.0;
            for _ in 0..5 { for _ in 0..k { go(&p1, one); } }
            let t = Instant::now(); for _ in 0..50 { for _ in 0..k { go(&p1, one); } }
            let seq = t.elapsed().as_micros() as f64 / 50.0;
            println!("{n},{k},{fused:.1},{seq:.1},{:.2}x", seq / fused.max(0.05));
        }
    }
}
