coverage:
    cargo llvm-cov --lcov --output-path=lcov.info --ignore-filename-regex tests\.rs
    genhtml lcov.info --dark-mode --flat --missed --output-directory target/coverage_html

# Run e2e tests with coverage (includes instrumented library coverage)
coverage-e2e:
    source <(cargo llvm-cov show-env --export-prefix) && \
    cargo build && \
    sudo cp target/debug/libvapoursynth_zoomvtools.so /usr/lib/vapoursynth/ && \
    cargo llvm-cov test --features e2e --no-clean --lcov --output-path=lcov.info --ignore-filename-regex tests\.rs && \
    genhtml lcov.info --dark-mode --flat --missed --output-directory target/coverage_html

codecov:
    cargo llvm-cov --codecov --output-path codecov.json --ignore-filename-regex tests\.rs

codecov-upload:
    just codecov
    codecov --token "$ZOOMV_CODECOV_TOKEN" --file codecov.json --required

# Install optimized release build
install:
    cargo build --release
    sudo cp target/release/libvapoursynth_zoomvtools.so /usr/lib/vapoursynth/

# Install debug build
install-debug:
    cargo build
    sudo cp target/debug/libvapoursynth_zoomvtools.so /usr/lib/vapoursynth/

# Run the benchmark suite
bench:
    cargo bench --features bench

# Build the benchmark suite without running it
bench-build:
    cargo bench --features bench --no-run

# Pre-commit action to verify code quality
precommit:
    cargo fmt
    cargo clippy
    cargo test
    just bench-build

# Run end-to-end tests (requires C MVTools)
e2e:
    cargo test --features e2e

# Check if C MVTools is installed
check-mvtools:
    @python3 -c "import vapoursynth as vs; core = vs.core; core.mv.Super" 2>/dev/null && echo "C MVTools is installed" || echo "C MVTools NOT found - install vapoursynth-plugin-mvtools"

# Install debug build and run e2e tests
e2e-dev:
    just install-debug
    cargo test --features e2e -- --nocapture
