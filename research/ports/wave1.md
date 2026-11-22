# Author-plus-critic port redo, wave 1

Thirty corpus ports rewritten from scratch in CP Lang 0.2.2, each one held to
four gates on the Rust `cpl`: `cpl check`, `cpl build` compiled with
`g++ -fsanitize=address,undefined`, `cpl run`, and a differential run of both
against the original accepted C++ on random inputs.

The ports live where they always did: `proto/ports/<name>.cp`, and
`proto/ports/hillclimb_dp_graph/<name>.cp` for `apio09p3`, `coi06p2`, `dps`
and `ioi11p4`. This directory holds only the harness: one input generator per
port in `gen/`, and `gate.sh`.

    cargo build --release -p cpl-cli
    proto/ports/wave1/gate.sh dpz 100      # one port, 100 random cases
    for g in proto/ports/wave1/gen/*.py; do proto/ports/wave1/gate.sh $(basename $g .py); done

The wave: aac4p2 apio09p3 btoi17p3 ccc10s4 ccc17s4 ccc18s5 ccc19s5 ccc21s3
ccc21s5 ccc22s4 ceoi18p1 cheerio1s3 coi06p2 dmopc21c2p1 dmpg16s5 dpe dpg dph
dps dpz ioi10p3 ioi11p4 noi14p4 noip04p2 noip07p1 noip07p2 noip09p2 noip09p3
noip10p3 noip14p3.

`ccc10s4`'s generator keeps the pens connected: the original C++ sums a
spanning forest when they are not, and the port insists on a spanning tree.

## Blind critic

An independent session that never saw the author's reasoning then attacked
every line. `CRITIC.md` is its report: 30 rewrites that pass the gates, all
applied here, 39 cleaner spellings the Rust `cpl` or the stdlib does not
allow yet, each with the exact error, and one bug: the first `dpg` recursed
with `@memo` and its emitted C++ overflowed the stack on a 100000-vertex
chain, so it now walks a topological order.

## Blocked on the Rust compiler

Only ports whose clean 0.2.2 form passes the Rust gates today are in the wave;
nothing was worked around. These ports and spellings were left for the Rust
graduation pass, each line naming what `cpl` rejects:

- dpr dpt dpy yac1p1 noip14p2: sum(...) over Mod<P> (E0309 `sum` adds numbers)
- dpp noi01p5: product(...) missing
- dpo: read() of Bool
- any: a.items on arrays/lists (E0203); print(x, none: ..); `if let` in comprehension filter; max(xs) over [D]?Int; Range.rev stored in a let; arrays over 1..=n ranges (E0501); bsearch_first(I, ..) on a domain
- mccc4j5: read() of a record with a List<V> field
- nccc10s2: Perm<L>
- any: one-line `enum A: X(..) | Y(..)` with payloads; `{}` literal for Map/Set; prefix1(a) not in prelude; dijkstra/bfs over a plain [V]List<(to:,w:)> adjacency
- ccc21s2 (and cco18p1, noi02p1): read() of a @tags enum
- any: dependent bounds inside one tuple read `let (n, k): (1..=1e5, 1..=n) = read()`
- noip06p2 yac6p2: read(none: 0) into ?V (E0310)
- any: `opt += x` on ?Int (tropical `+` works, `+=` E0309); Graph.undirected(edges) needs explicit Graph<V, E> type args and rejects edge records with extra fields (weights)
- any: `choose` needs `use std.math.{choose}`, `desc` needs `use std.algebra.{desc}`, `bsearch_first` needs `use std.seq.{bsearch_first}` (prelude gaps); window ranges need `i.l..i.r(h)` (no `Row.windows(h)`)
- ccc23s4: clean rewrite passes the Rust gates, but crates/cpl-verify's bench test runs this file through the Python prototype, which rejects named arguments on dijkstra(from:, next:, w:) ("unsupported expression kwarg")
