# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

vapoursynth-zoomvtools is a VapourSynth plugin implementing motion vector estimation tools in Rust. It provides motion analysis and super-resolution functionality for video processing, designed as a Rust rewrite of MVTools for performance and safety.

## Build Commands

```bash
# Build the plugin
cargo build --release

# Install the plugin to system VapourSynth directory
just install

# Install debug version
just install-debug

# Run tests
cargo test

# Run a specific test
cargo test <test_name>

# Run with ignored tests
cargo test -- --ignored

# Format code
cargo fmt

# Run linter
cargo clippy

# Pre-commit checks (format, lint, coverage, bench build)
just precommit
```

## Testing and Coverage

```bash
# Generate coverage report (lcov.info + HTML report in target/coverage_html/)
just coverage

# Generate codecov JSON
just codecov

# Upload to codecov (requires ZOOMV_CODECOV_TOKEN)
just codecov-upload
```

## End-to-End Testing

E2E tests compare the Rust implementation against the original C MVTools plugin. Requires C MVTools to be installed.

```bash
# Check if C MVTools is installed
just check-mvtools

# Run end-to-end tests
just e2e

# Install debug build and run e2e tests (development workflow)
just e2e-dev

# Run e2e tests with coverage
just coverage-e2e

# Run specific e2e test with output
cargo test --features e2e test_name -- --nocapture
```

## Benchmarking

Benchmarks use Criterion and require the `bench` feature flag:

```bash
# Run all benchmarks
just bench

# Build benchmarks without running
just bench-build

# Run specific benchmark
cargo bench --features bench --bench <benchmark_name>
```

Available benchmarks: `average`, `pad`, `reduce`, `refine`, `util`

### Comparative Benchmarks (vs C MVTools)

`scripts/benchmark.py` uses `hyperfine` to compare zoomv (Rust) against mv (C) filter performance. Requires `hyperfine`, `vspipe`, and C MVTools installed.

```bash
# Run all Super parameter sets at both bit depths
python scripts/benchmark.py super

# Run a specific test at a specific bit depth
python scripts/benchmark.py super --test pel4 --bits 8
```

New filters can be added to the `FILTERS` dict in the script with a `generate_script` function and `tests` dict.

## Architecture

### VapourSynth Plugin Structure

The plugin exports two main filters via `lib.rs`:

- **Super**: Creates hierarchical multi-resolution representations with optional padding and sub-pixel precision (`src/mv_super.rs`)
- **Analyse**: Performs motion vector estimation on super clips (`src/mv_analyse.rs`)

Both filters integrate with VapourSynth's API through the `vapoursynth` crate and use the `make_filter_function!` and `export_vapoursynth_plugin!` macros.

### Motion Vector Estimation Pipeline

The core motion estimation is organized hierarchically:

1. **MVGroupOfFrames** (`src/mv_gof.rs`): Manages groups of frames for temporal motion analysis
2. **GroupOfPlanes** (`src/group_of_planes.rs`): Container for multi-level plane hierarchies
3. **PlaneOfBlocks** (`src/plane_of_blocks.rs`): The main workhorse - performs block matching and motion search on individual planes. Contains complex search algorithms (exhaustive, diamond, hexagon, UMH) and predictor logic
4. **MVFrame** (`src/mv_frame.rs`): Represents motion vector data for a single frame
5. **MVPlane** (`src/mv_plane.rs`): Plane-level motion vector storage and operations

### Key Data Structures

- **MotionVector** (`src/mv.rs`): Compact representation of motion vectors with SAD/cost metrics
- **MVAnalysisData**: Metadata structure embedded in output frames describing the motion search configuration
- **Subpel** (`src/params.rs`): Sub-pixel precision levels (Full/Half/Quarter)
- **SearchType** (`src/params.rs`): Motion search algorithms (Exhaustive, Logarithmic, Diamond, Hexagon, UMH, etc.)

### Interpolation and Downsampling

Multi-resolution support uses specialized filters:

- **reduce/** (`src/reduce/`): Downsampling filters (Average, Triangle, Bilinear, Quadratic, Cubic)
- **refine/** (`src/refine/`): Sub-pixel interpolation (Bicubic, Bilinear, Wiener)
- **pad.rs**: Edge padding strategies for block matching at frame boundaries

### Performance-Critical Components

Performance-sensitive code in `src/util/`:

- **sad/** (`src/util/sad/mod.rs`): Sum of Absolute Differences calculation with size-specific optimizations
- **satd/** (`src/util/satd/mod.rs`): Sum of Absolute Transformed Differences (uses Hadamard transform via DCT)
- **luma/** (`src/util/luma/mod.rs`): Luma sum calculations
- **Pixel trait** (`src/util/mod.rs`): Generic trait over `u8` and `u16` pixel types with arithmetic operations

AVX2 detection via `cpufeatures` crate is available but SIMD implementations may not be complete.

### Feature Flags

- `bench`: Exposes internal modules as `pub` for benchmarking
- `e2e`: Enables end-to-end tests that compare against C MVTools
- `no_simd`: Disables SIMD optimizations (testing/debugging only - "will make me sad" per Cargo.toml)

## Development Notes

### SIMD Code

- SIMD intrinsics are the primary use of `unsafe` code
- The linter allows `unsafe_op_in_unsafe_fn` specifically for SIMD operations
- The `no_simd` feature exists for testing but should not be used in production

### Linter Configuration

Many clippy overrides at the top of `lib.rs`.

Notable overrides:

- We are using the `mod_module_files` pattern, so top-level modules remain e.g. `src/util.rs` and do _not_ get moved to `mod.rs` in a subfolder.

### Testing Structure

Tests are co-located with modules:

- Unit tests in `#[cfg(test)] mod tests` within each module, in separate files (e.g., `src/pad/tests.rs`, `src/refine/tests.rs`)
- Integration tests use `#[macro_use] mod tests` in `src/tests.rs`
- End-to-end tests require the C version of the plugin to be installed, and intend to verify our results are equivalent to theirs

Use parameterized tests via the `parameterized` crate to simplify validating functions that have many possible values to test, and to validate the most commonly processed image formats, which are "vs.YUV420P8", "vs.YUV420P10", and "vs.YUV420P16".

The `parameterized` crate uses named parameter columns (not tuples). For multi-dimensional parameters, use separate columns that get zipped: `#[parameterized(w = { 4, 8 }, h = { 4, 8 })] fn test(w: usize, h: usize)` — not `width_height = { (4, 4), (8, 8) }`.

### Motion Search Implementation Details

The `PlaneOfBlocks` struct contains the core search logic. Key aspects:

- Supports multiple search patterns (exhaustive, diamond, hexagon, UMH with increasing complexity)
- Uses predictors from neighboring blocks and previous frames
- Implements hierarchical search from coarse to fine levels
- DCT-based SAD calculation option via `dct.rs` for handling luma changes
- Bad block handling with extended search radius
- Global motion estimation support
- Block subdivision with median motion vectors
