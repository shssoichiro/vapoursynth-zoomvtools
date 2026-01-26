#!/usr/bin/env python3
"""Benchmark zoomv (Rust) against mv (C) using hyperfine."""

import argparse
import os
import subprocess
import sys
import tempfile

SOURCE_FILES = {
    8: "test_files/flower_8bit.mkv",
    10: "test_files/flower_10bit.mkv",
}


def generate_super_script(source_path, params):
    params_str = ", ".join(f"{k}={v}" for k, v in params.items())
    return f"""\
import vapoursynth as vs

core = vs.core

clip = core.ffms2.Source(source="{source_path}")

clip_mv = clip.mv.Super({params_str})
clip_zoom = clip.zoomv.Super({params_str})

clip_mv.set_output(0)
clip_zoom.set_output(1)
"""


FILTERS = {
    "super": {
        "generate_script": generate_super_script,
        "tests": {
            "default": {},
            "pel1": {"pel": 1},
            "pel4": {"pel": 4},
            "sharp_bilinear": {"sharp": 0},
            "sharp_bicubic": {"sharp": 1},
            "sharp_wiener": {"sharp": 2},
            "rfilter_average": {"rfilter": 0},
            "rfilter_cubic": {"rfilter": 4},
            "no_chroma": {"chroma": 0},
            "small_pad": {"hpad": 4, "vpad": 4},
            "large_pad": {"hpad": 32, "vpad": 32},
        },
    },
}


def main():
    parser = argparse.ArgumentParser(
        description="Benchmark zoomv (Rust) against mv (C) using hyperfine."
    )
    parser.add_argument(
        "filter",
        help=f"Which filter to benchmark. Available: {', '.join(FILTERS)}",
    )
    parser.add_argument(
        "--test",
        help="Run a specific named parameter set. If omitted, runs all.",
    )
    parser.add_argument(
        "--bits",
        default="all",
        help="Bit depth to test: 8, 10, or all (default: all).",
    )
    args = parser.parse_args()

    if args.filter not in FILTERS:
        print(
            f"Error: Unknown filter '{args.filter}'. "
            f"Available filters: {', '.join(FILTERS)}",
            file=sys.stderr,
        )
        sys.exit(1)

    filter_def = FILTERS[args.filter]
    filter_name = args.filter.capitalize()
    tests = filter_def["tests"]
    generate_script = filter_def["generate_script"]

    if args.test is not None:
        if args.test not in tests:
            print(
                f"Error: Unknown test '{args.test}' for filter '{args.filter}'. "
                f"Available tests: {', '.join(tests)}",
                file=sys.stderr,
            )
            sys.exit(1)
        tests = {args.test: tests[args.test]}

    if args.bits == "all":
        bit_depths = [8, 10]
    elif args.bits in ("8", "10"):
        bit_depths = [int(args.bits)]
    else:
        print(
            f"Error: Invalid --bits value '{args.bits}'. Must be 8, 10, or all.",
            file=sys.stderr,
        )
        sys.exit(1)

    for bits in bit_depths:
        source_path = os.path.abspath(SOURCE_FILES[bits])
        if not os.path.exists(source_path):
            print(
                f"Error: Source file not found: {source_path}",
                file=sys.stderr,
            )
            sys.exit(1)

        for test_name, params in tests.items():
            print(f"\n=== {filter_name}: {test_name} ({bits}-bit) ===\n", flush=True)

            script_content = generate_script(source_path, params)
            tmp = tempfile.NamedTemporaryFile(suffix=".vpy", delete=False, mode="w")
            try:
                tmp.write(script_content)
                tmp.close()

                subprocess.run(
                    [
                        "hyperfine",
                        "-w",
                        "1",
                        "-n",
                        f"C (mv.{filter_name})",
                        f"vspipe -o 0 -p {tmp.name} --",
                        "-n",
                        f"Rust (zoomv.{filter_name})",
                        f"vspipe -o 1 -p {tmp.name} --",
                    ],
                    check=True,
                )
            finally:
                os.unlink(tmp.name)


if __name__ == "__main__":
    main()
