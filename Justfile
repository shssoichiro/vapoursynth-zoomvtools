coverage:
    cargo llvm-cov --ignore-filename-regex tests\.rs

lcov:
    cargo llvm-cov --lcov --output-path=lcov.info --ignore-filename-regex tests\.rs

codecov:
    cargo llvm-cov --codecov --output-path codecov.json --ignore-filename-regex tests\.rs
    
codecov-upload:
    just codecov && codecov --token "$ZOOMV_CODECOV_TOKEN" --file codecov.json --required

install:
    cargo build --release && sudo cp target/release/libvapoursynth_zoomvtools.so /usr/lib/vapoursynth/

install-debug:
    cargo build && sudo cp target/debug/libvapoursynth_zoomvtools.so /usr/lib/vapoursynth/

bench:
    cargo bench --features bench

bench-build:
    cargo bench --features bench --no-run

precommit:
    cargo fmt && cargo clippy && just lcov && just bench-build

