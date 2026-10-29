//! Build script: compile the C++ hot loop with the system compiler.
//! Flags mirror the metal_cpp bench (native arch, -O3). No build-time deps.

fn main() {
    let mut build = cc::Build::new();
    build
        .cpp(true)
        .file("cpp-bridge/hotloop.cpp")
        .flag("-std=c++17")
        .flag("-O3")
        .flag_if_supported("-march=native")
        .flag_if_supported("-ffast-math")
        .compile("hm_hotloop");
    println!("cargo:rerun-if-changed=cpp-bridge/hotloop.cpp");
}
