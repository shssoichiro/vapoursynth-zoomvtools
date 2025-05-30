coverage:
    cargo llvm-cov --ignore-filename-regex tests\.rs

lcov:
    cargo llvm-cov --lcov --output-path=lcov.info --ignore-filename-regex tests\.rs

install:
    cargo build --release && sudo cp target/release/libvapoursynth_zoomvtools.so /usr/lib/vapoursynth/

install-debug:
    cargo build && sudo cp target/debug/libvapoursynth_zoomvtools.so /usr/lib/vapoursynth/