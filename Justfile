coverage:
    cargo llvm-cov --lcov --output-path=lcov.info --ignore-filename-regex tests\.rs
    genhtml lcov.info --dark-mode --flat --missed --output-directory target/coverage_html

coverage-e2e:
    cargo llvm-cov --features e2e --lcov --output-path=lcov.info --ignore-filename-regex tests\.rs
    genhtml lcov.info --dark-mode --flat --missed --output-directory target/coverage_html

codecov:
    cargo llvm-cov --codecov --output-path codecov.json --ignore-filename-regex tests\.rs

codecov-upload:
    just codecov
    codecov --token "$ZOOMV_CODECOV_TOKEN" --file codecov.json --required

install:
    cargo build --release
    sudo cp target/release/libvapoursynth_zoomvtools.so /usr/lib/vapoursynth/

install-debug:
    cargo build
    sudo cp target/debug/libvapoursynth_zoomvtools.so /usr/lib/vapoursynth/

bench:
    cargo bench --features bench

bench-build:
    cargo bench --features bench --no-run

precommit:
    cargo fmt
    cargo clippy
    just lcov
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
