# End-to-End Testing

This directory contains end-to-end (e2e) tests that compare the Rust implementation against the original C MVTools plugin.

## Prerequisites

### 1. C MVTools Installation

The e2e tests require the C version of MVTools to be installed:

**Arch Linux:**
```bash
pacman -S vapoursynth-plugin-mvtools
```

**Ubuntu/Debian:**
```bash
apt-get install vapoursynth-mvtools
```

**From source:**
```bash
git clone https://github.com/dubhater/vapoursynth-mvtools
cd vapoursynth-mvtools
./autogen.sh && ./configure && make && sudo make install
```

### 2. Verify Installation

Check that C MVTools is available:
```bash
just check-mvtools
```

You should see: `C MVTools is installed`

### 3. Install Rust Plugin

Install the Rust implementation you want to test:
```bash
# For debug build
just install-debug

# For release build
just install
```

## Running Tests

### Run all e2e tests:
```bash
cargo test --features e2e
```

Or using the justfile:
```bash
just e2e
```

### Run with verbose output:
```bash
just e2e-verbose
```

### Run specific test:
```bash
cargo test --features e2e test_super_8bit_yuv420_default_params
```

### Run with ignored tests (performance tests):
```bash
cargo test --features e2e -- --ignored --nocapture
```

### Install and run in one command:
```bash
just e2e-dev
```

## Test Organization

### Test Files

- **`e2e_super.rs`** - Tests for the Super filter
  - Pixel-perfect comparison across different formats (YUV420, YUV422, YUV444, GRAY)
  - Property verification (Super_height, Super_hpad, Super_vpad, Super_pel, Super_modeyuv, Super_levels)
  - 8-bit and 16-bit depth testing
  - Custom parameter testing (hpad, vpad, pel, levels)
  - Performance benchmarks (marked with `#[ignore]`)

- **`e2e_analyse.rs`** - Tests for the Analyse filter
  - Motion vector comparison with tolerance
  - Different search algorithms (Onetime, Nstep, Exhaustive, Hex2, UMH, Horizontal, Vertical)
  - Backward/forward motion testing
  - Different block sizes (4, 8, 16, 32)
  - 8-bit and 16-bit depth testing
  - Performance benchmarks (marked with `#[ignore]`)

- **`common/`** - Shared utilities
  - `detection.rs` - C MVTools detection and error handling
  - `script_gen.rs` - VapourSynth script generation
  - `comparison.rs` - Frame and property comparison utilities
  - `perf.rs` - Performance measurement utilities

## Test Utilities

### Script Generation

The `script_gen` module creates VapourSynth Python scripts that apply both C and Rust filters:

```rust
let clip_config = TestClipConfig {
    width: 640,
    height: 480,
    format: "vs.YUV420P8",
    length: 10,
    content_type: ClipContentType::Gradient,
};

let params = FilterParams::default();
let script = generate_comparison_script(&clip_config, &params, None);
```

#### Available Content Types

- `Blank` - Solid black clip
- `Gradient` - Horizontal gradient
- `Checkerboard` - Checkerboard pattern
- `MovingBox { speed_x, speed_y }` - Moving white box on black background
- `Noise { seed }` - Random noise with specific seed

### Frame Comparison

Compare pixel data with configurable tolerance:

```rust
let config = ComparisonConfig {
    pixel_tolerance: 1.0,           // Allow 1-bit difference per pixel
    mean_tolerance: 0.5,             // Mean diff must be < 0.5
    max_different_pixels: 0.1,       // At most 0.1% pixels can differ
};

assert_frames_match::<u8>(&c_frame, &r_frame, &config, "context")?;
```

For exact matching (default):
```rust
let config = ComparisonConfig::default(); // All tolerances = 0
```

### Motion Vector Comparison

Compare motion vectors with SAD tolerance:

```rust
let c_vectors = c_frame.props().get_data("MVTools_vectors")?;
let r_vectors = r_frame.props().get_data("MVTools_vectors")?;

compare_motion_vectors(c_vectors, r_vectors, 100)?; // Allow 100 SAD difference
```

### Property Comparison

Compare frame properties:

```rust
// Compare all properties
compare_frame_properties(&c_frame, &r_frame, &[])?;

// Ignore specific properties
compare_frame_properties(&c_frame, &r_frame, &["_Matrix", "_ChromaLocation"])?;
```

### Performance Measurement

Measure and compare processing speed:

```rust
let c_perf = measure_filter_performance(&c_node, 50, "C MVTools", "Super")?;
let r_perf = measure_filter_performance(&r_node, 50, "Rust zoomv", "Super")?;

println!("{}", compare_performance(&c_perf, &r_perf));
```

Output example:
```
Performance comparison for Super:
C MVTools:  45.23 fps (22.11ms/frame)
Rust zoomv: 52.17 fps (19.17ms/frame)
Speedup: 1.15x (Rust faster)
```

## Writing New Tests

### Basic Test Structure

```rust
#[test]
#[cfg(feature = "e2e")]
fn test_my_feature() -> Result<()> {
    require_mvtools!(); // Ensure C MVTools is available

    // 1. Configure test clip
    let clip_config = TestClipConfig {
        width: 320,
        height: 240,
        format: "vs.YUV420P8",
        length: 10,
        content_type: ClipContentType::MovingBox {
            speed_x: 2,
            speed_y: 1,
        },
    };

    // 2. Configure filter parameters
    let super_params = FilterParams::default();
    let analyse_params = FilterParams {
        blksize: Some(16),
        search: Some(3), // Exhaustive
        ..Default::default()
    };

    // 3. Generate comparison script
    let script = generate_comparison_script(
        &clip_config,
        &super_params,
        Some(&analyse_params),
    );

    // 4. Create environment and get outputs
    let env = Environment::from_script(&script)?;
    let (c_node, _) = env.get_output(0)?; // C implementation
    let (r_node, _) = env.get_output(1)?; // Rust implementation

    // 5. Compare outputs
    for n in 0..clip_config.length {
        let c_frame = c_node.get_frame(n)?;
        let r_frame = r_node.get_frame(n)?;

        // Compare motion vectors
        let c_vectors = c_frame.props().get_data("MVTools_vectors")?;
        let r_vectors = r_frame.props().get_data("MVTools_vectors")?;
        compare_motion_vectors(c_vectors, r_vectors, 100)?;
    }

    Ok(())
}
```

### Performance Test Structure

Performance tests should be marked with `#[ignore]`:

```rust
#[test]
#[cfg(feature = "e2e")]
#[ignore] // Performance test - run explicitly
fn test_my_performance() -> Result<()> {
    require_mvtools!();
    // ... setup ...

    let c_perf = measure_filter_performance(&c_node, 50, "C MVTools", "MyFilter")?;
    let r_perf = measure_filter_performance(&r_node, 50, "Rust zoomv", "MyFilter")?;

    println!("{}", compare_performance(&c_perf, &r_perf));
    Ok(())
}
```

## Troubleshooting

### "C MVTools not detected"

**Problem:** Test fails with MVTools installation error.

**Solution:**
1. Install C MVTools (see Prerequisites)
2. Verify with `just check-mvtools`
3. Check that `libmvtools.so` exists in `/usr/lib/vapoursynth/`

### "Property not found"

**Problem:** Property comparison fails because properties differ.

**Solution:**
- Add the property to the ignore list:
  ```rust
  compare_frame_properties(&c_frame, &r_frame, &["_Matrix"])?;
  ```
- Or investigate why the property is missing/different

### Motion vector mismatches

**Problem:** Motion vectors don't match exactly.

**Solution:**
- Increase SAD tolerance:
  ```rust
  compare_motion_vectors(c_vectors, r_vectors, 200)?; // Higher tolerance
  ```
- Different search algorithms may find different (but equally valid) vectors
- Consider testing that vectors are "good enough" rather than identical

### Frame dimension mismatch

**Problem:** C and Rust output different frame dimensions.

**Solution:**
- Check Super filter parameters (hpad, vpad, pel)
- Verify both implementations handle parameters the same way
- This likely indicates a bug in the Rust implementation

## CI Integration

E2E tests are **not run in CI by default** because they require C MVTools to be installed.

To enable in CI:
1. Install C MVTools in the CI environment
2. Add to workflow:
   ```yaml
   - name: Run E2E tests
     run: cargo test --features e2e
   ```

## Expected Test Status

### Initial Implementation

The e2e test framework is designed to work even if some tests fail:

- **Super filter tests**: Expected to mostly pass (implementation is more complete)
- **Analyse filter tests**: May have failures (implementation still under development)

The goal of the initial implementation is to have the testing infrastructure operational, not to have all tests passing. As development continues, more tests should pass.

### Debugging Failed Tests

When a test fails:

1. Run with `--nocapture` to see detailed output:
   ```bash
   cargo test --features e2e test_name -- --nocapture
   ```

2. Check the failure details - the comparison utilities provide detailed diff information

3. Investigate the Rust implementation to find the discrepancy

4. Fix the bug or adjust tolerance if the difference is acceptable

## Contributing

When adding new tests:

1. Follow the existing test patterns
2. Use descriptive test names
3. Add comments explaining what's being tested
4. Set appropriate comparison tolerances
5. Mark performance tests with `#[ignore]`
6. Update this README if adding new test utilities
