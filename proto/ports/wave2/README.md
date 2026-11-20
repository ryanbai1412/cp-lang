# Author-plus-critic port redo, wave 2

Thirty more corpus ports rewritten from scratch in CP Lang 0.2.2, each one
held to four gates on the Rust `cpl`: `cpl check`, `cpl build` compiled with
`g++ -fsanitize=address,undefined`, `cpl run`, and a differential run of both
against the original accepted C++ on 100 random inputs.

Unlike wave 1, the rewrites live here, next to the harness, so the older
drafts in `proto/ports/` stay untouched for comparison: `<name>.cp` is the
port, `gen/<name>.py` its input generator, and `gate.sh` runs the gates
against `proto/ports/<name>_orig.cpp` (or `hillclimb_dp_graph/`).

    cargo build --release -p cpl-cli
    proto/ports/wave2/gate.sh dpv 100      # one port, 100 random cases
    for g in proto/ports/wave2/gen/*.py; do proto/ports/wave2/gate.sh $(basename $g .py); done

The wave: ccc18s4 ccc20s2 ccc21s2 cco18p1 dmopc21c5p1 dpk dpl dpm dpn dpo dpp
dpr dpt dpu dpv dpx dpy ioi00p1 ioi04p4 ioi94p1 noip00p1 noip01p2 noip04p1
noip04p3 noip06p1 noip06p2 noip14p2 noip14p5 noip18p2 yac1p1.

None of the 30 overlaps wave 1 or `proto/ports/ecnerwala/`.

## Where the Rust `cpl` forced a longer spelling

Every port passes the gates as written, but these lines are not the 0.2.2 the
cheat sheet promises. Each is the shortest form the Rust `cpl` accepts today;
the wanted form is on the right.

- dpr dpt dpy dpo yac1p1 noip14p2: an accumulating `for` instead of
  `sum(...)` over `Mod<P>` (E0309 "`sum` adds numbers").
- dpr: a three-loop `mul` instead of `[V, V] (i, j) => sum(...)`.
- dpm: a hand-slid window instead of `prefix(ways)` (E0302, `prefix` takes
  `[D]Int`).
- dpt dpy noip04p1 noip06p2: an index loop instead of `for (i, x) in a.items`
  (E0203 on `Str`, `List` and domain arrays).
- dpp dpv noip14p2 noip14p5: a named `domain E` for the edge list;
  `[0..m](u: V, v: V)` does not infer `Graph`'s edge domain (E0201 `E`).
- noip14p2 noip14p5: `for (v, _) in g.out(u)` instead of `for v in g(u)`.
- noip14p5 ccc20s2: `use std.graph.{bfs}`, and `bfs(x => leads[x], ...)`
  instead of passing the adjacency array.
- noip14p5: `print(x, none: "-1")`; `none:` takes a `Str` only.
- ccc21s2 ccc20s2 noip04p1: 0-based arrays with `- 1` / `+ 1` at the use,
  because `[1..=n]T` is rejected (E0501).
- ioi00p1 noip04p3 cco18p1: plain `0..n` instead of a `domain`, because
  `l + 1..n` over a domain is a range of `?I` (E0302), and `..i` has no
  `for` support yet (E0501).
- noip06p2: `bundles = bundles + [...]` instead of `+=` on a `List` (E0309).
- noip14p2: `sorted(xs, by: x => -x)` instead of `by: desc`.

## Blind critic

`CRITIC.md` is the report of an independent session that saw only the ports
and the gate, never the author's reasoning.

## Left untested

`ccc18s4`'s generator stops at n = 1e7: at n near 1e9 the emitted C++ is fine
but `cpl run` takes about 26 s on the `@memo` recursion (some 5e6 loop steps),
past the gate's 20 s limit. Every other port is tested over its generator's
full range, which is small random inputs rather than the judges' maximum sizes.
