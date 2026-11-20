# ecnerwala/contest-programming ports

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
`ACM/UCup5/Stage05-Chengdu/E/E.cpp`. Wave 2 adds stages 3 (`ucup5s03f`,
Gorkovskaya), 2 (`ucup5s02e`, EPS) and 1 (`ucup5s01a`, Nizhny Novgorod).

Two of the originals (`cf2261g`, `cf2261h`) use C++23 deducing `this`, which
g++ 11 rejects; `gate.sh` falls back to `zig c++` (clang 20) for those.

`ucup5s05b_orig.cpp` starts with a short `wala::Point` shim, marked as not
upstream, because the archived file does not compile without it.

What was skipped and every piece of friction found is in
`research/ecnerwala-corpus-wave-1.md` (13 ports, upstream 2026-10-17 and
later) and `research/ecnerwala-corpus-wave-2.md` (21 ports, back to August
2026).

Five power-series ports (`ucup5s06a`, `ucup5s05f`, `ucup5s03a`, `cf2268e`,
`cf2262e`) import `examples/local-lib/math/poly.cp` as `lib.math.poly`:
`lib.toml` here names the library and `math` is a link to that folder.
`ucup5s06a` needs `INTERP_TIMEOUT=200 proto/ports/ecnerwala/gate.sh ucup5s06a`,
because `cpl run` takes 78 s on its n = 100000 sample. What still blocks the
rest of the corpus, with counts, is in `research/ecnerwala-100pct-roadmap.md`.
