// Build script for CUDA Rust hyperbolic KV benchmark
//
// Handles CUDA kernel compilation for both cutile-rs (Tile) and cuda-oxide (SIMT) tracks

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=CUDA_PATH");
    println!("cargo:rerun-if-env-changed=CUDA_HOME");

    // Detect CUDA toolkit
    let cuda_path = detect_cuda();
    println!("cargo:rustc-env=CUDA_PATH={}", cuda_path.display());

    // Set library search path
    println!(
        "cargo:rustc-link-search=native={}/lib64",
        cuda_path.display()
    );
    println!("cargo:rustc-link-search=native={}/lib", cuda_path.display());

    // Link CUDA runtime (Linux only; macOS validates semantics CPU-side)
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "linux" {
        println!("cargo:rustc-link-lib=dylib=cudart");
        println!("cargo:rustc-link-lib=dylib=cuda");
    }

    // For cutile-rs (Tile track), JIT compilation happens at runtime
    if cfg!(feature = "cutile-track") {
        println!("cargo:rustc-cfg=cutile_available");
    }

    // Verify nvcc is available for C++ baseline comparison
    if let Ok(output) = Command::new("which").arg("nvcc").output() {
        if output.status.success() {
            let nvcc_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!("cargo:rustc-env=NVCC_PATH={}", nvcc_path);
        }
    }
}

fn detect_cuda() -> PathBuf {
    // Check environment variables first
    if let Ok(path) = env::var("CUDA_PATH") {
        return PathBuf::from(path);
    }
    if let Ok(path) = env::var("CUDA_HOME") {
        return PathBuf::from(path);
    }

    // Common locations
    let candidates = ["/usr/local/cuda", "/opt/cuda", "/usr/lib/cuda"];

    for candidate in &candidates {
        let path = PathBuf::from(candidate);
        if path.exists() {
            return path;
        }
    }

    // Fallback
    PathBuf::from("/usr/local/cuda")
}
