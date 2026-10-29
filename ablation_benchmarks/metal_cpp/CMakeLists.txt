cmake_minimum_required(VERSION 3.28)
project(hyperbolic_kv_bench_cpp LANGUAGES CXX CUDA)

# C++20 standard
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# CUDA configuration
set(CMAKE_CUDA_STANDARD 17)
set(CMAKE_CUDA_STANDARD_REQUIRED ON)
set(CMAKE_CUDA_ARCHITECTURES "90;100;120")  # H100, B200, RTX 5090

# Build configuration
set(CMAKE_BUILD_TYPE Release)
set(CMAKE_CXX_FLAGS_RELEASE "-O3 -march=native -ffast-math -flto=thin")
set(CMAKE_CUDA_FLAGS_RELEASE "-O3 -arch=native -ffast-math -flto=thin")

# Output directories
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin)
set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)
set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)

# Find CUDA
find_package(CUDA REQUIRED)

# Include directories
include_directories(${CMAKE_CURRENT_SOURCE_DIR}/include)

# Source files
set(SOURCES
    src/hyperbolic.cpp
    src/vptree.cpp
    src/picard.cu
    src/toroidal.cpp
    src/workload.cpp
    src/main.cpp
)

# Benchmark executable
add_executable(hyperbolic_kv_bench_cpp ${SOURCES})

# Link libraries
target_link_libraries(hyperbolic_kv_bench_cpp PRIVATE
    ${CUDA_LIBRARIES}
    pthread
    dl
    rt
)

# Compile options for hand-rolled SIMD
target_compile_options(hyperbolic_kv_bench_cpp PRIVATE
    -Wall -Wextra -Wpedantic
    -DQ32_31_SHIFT=31
    -DQ32_31_SCALE=2147483648
)

# CUDA-specific compile options for PTX/Tensor Core kernels
set_source_files_properties(src/picard.cu PROPERTIES
    CUDA_SEPARABLE_COMPILATION ON
)

# Install
install(TARGETS hyperbolic_kv_bench_cpp DESTINATION bin)

# Custom target for NCU profiling
add_custom_target(ncu_profile
    COMMAND ncu --target-processes all --set full --log-file ${CMAKE_BINARY_DIR}/ncu_log.txt ./bin/hyperbolic_kv_bench_cpp
    DEPENDS hyperbolic_kv_bench_cpp
    WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
    COMMENT "Running NCU profiling"
)

# Custom target for build timing
add_custom_target(build_time
    COMMAND time cmake --build . --target hyperbolic_kv_bench_cpp
    COMMENT "Measuring build time"
)
