# ecnerwala/contest-programming ports, wave 1

Recent submissions from <https://github.com/ecnerwala/contest-programming>
(Andrew He's solutions, CC0), newest first, rewritten in CP Lang 0.2.2 and held
to the same four gates as `proto/ports/wave1/` on the Rust `cpl`: `cpl check`,
`cpl build` compiled with `g++ -fsanitize=address,undefined`, `cpl run`, and a
differential run of both against the original C++.

    cargo build --release -p cpl-cli
    proto/ports/ecnerwala/gate.sh agc079c 150     # one port, 150 random cases
    for f in proto/ports/ecnerwala/*.cp; do proto/ports/ecnerwala/gate.sh $(basename $f .cp); done

Each `NAME.cp` sits beside `NAME_orig.cpp`, the submission as committed
upstream. `samples/` holds the sample files upstream kept next to it, and
`gen/NAME.py SEED SIZE` prints one random input.

The port names map to upstream paths like this: `agc079a` is
`AtCoder/AGC079/A/A.cpp`, `cf2261g` is `CF/2261/G/G.cpp`, `ucup5s06g` is
`ACM/UCup5/Stage06-Hyakumanben/G/G.cpp`, and `ucup5s05e` is
`ACM/UCup5/Stage05-Chengdu/E/E.cpp`.

Two of the originals (`cf2261g`, `cf2261h`) use C++23 deducing `this`, which
g++ 11 rejects; `gate.sh` falls back to `zig c++` (clang 20) for those.

What was skipped and every piece of friction found is in
`research/ecnerwala-corpus-wave-1.md`.
