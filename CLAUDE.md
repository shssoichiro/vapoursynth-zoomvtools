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
# Generate coverage report
just coverage

# Generate HTML coverage report
just lcov

# Generate codecov JSON
just codecov

# Upload to codecov (requires ZOOMV_CODECOV_TOKEN)
just codecov-upload
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
- `no_simd`: Disables SIMD optimizations (testing/debugging only - "will make me sad" per Cargo.toml)

## Development Notes

### SIMD Code

- SIMD intrinsics are the primary use of `unsafe` code
- The linter allows `unsafe_op_in_unsafe_fn` specifically for SIMD operations
- The `no_simd` feature exists for testing but should not be used in production

### Linter Configuration

Notable clippy overrides in `Cargo.toml`:
- `manual_div_ceil = "allow"` - Manual division can have better performance
- `needless_range_loop = "allow"` - Sometimes explicit indexing is clearer for SIMD
- `too_many_arguments = "allow"` - Complex filter parameters require many arguments
- `undocumented_unsafe_blocks = "warn"` - Unsafe code should be documented
- `unnecessary_to_owned = "allow"` - False positive workaround

### Code Style

Uses custom rustfmt.toml with:
- `use_field_init_shorthand = true`
- `use_try_shorthand = true`

### Testing Structure

Tests are co-located with modules:
- Unit tests in `#[cfg(test)] mod tests` within each module
- Test-specific data in subdirectories (e.g., `src/pad/tests.rs`, `src/refine/tests.rs`)
- Integration tests use `#[macro_use] mod tests` in `src/tests.rs`

### Motion Search Implementation Details

The `PlaneOfBlocks` struct contains the core search logic. Key aspects:
- Supports multiple search patterns (exhaustive, diamond, hexagon, UMH with increasing complexity)
- Uses predictors from neighboring blocks and previous frames
- Implements hierarchical search from coarse to fine levels
- DCT-based SAD calculation option via `dct.rs` for handling luma changes
- Bad block handling with extended search radius
- Global motion estimation support
- Block subdivision with median motion vectors
