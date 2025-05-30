coverage:
    cargo llvm-cov

lcov:
    cargo llvm-cov --lcov --output-path=lcov.info

install:
    cargo build --release && sudo cp target/release/libvapoursynth_zoomvtools.so /usr/lib/vapoursynth/

install-debug:
    cargo build && sudo cp target/debug/libvapoursynth_zoomvtools.so /usr/lib/vapoursynth/