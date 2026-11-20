# Canonical ports that fail on HEAD

These are the canonical `proto/ports/*.cp` files (outside `ecnerwala/`, `scout_batch_*` and the critic folders) that had stale-port lint hits but failed `cpl check` or `cpl build` on HEAD at the start of stale-port cleanup Batch #12. There were 22 of them. Each entry gives the first compiler error, reproduced with `target/release/cpl check proto/ports/NAME.cp`.

## Batch #13: what is repaired

Batch #13, on top of Syntax batch 4 (`bcd873b`), repaired 6 of these ports. Each now passes `cpl check`, `cpl build` and `g++ -std=c++17 -fsyntax-only`, and has been removed from `PENDING`/`NOT_BUILT` in `crates/cpl-cli/tests/corpus.rs`. Five of them also print the same output as their `*_orig.cpp` on 40 random inputs each, from `wave2/gen/NAME.py` where one exists or a small generator written for the port otherwise.

- `noip06p2` and `yac6p3`: the removed `where` became `assert`. `yac6p3` is interactive, so it has no output diff.
- `dpy`: `where` became `assert sorted(walls).deduped().len == n`. The walls are sorted into a `[W]Cell` array.
- `cco16p2`: `where` became `assert`. The old `sort(seen, less: ...)` became `sort(mut seen, by: s => ByAngle(Pt(..)))`.
- `ioi11p2`: `Rooted(roads, root, weight: .len)` became `Rooted.weighted(Graph.undirected(roads), root, a => roads[a.e].len)`, and `tree.kids[v]` became `tree.children(v)`. `take(...)`, which `cpl run` can't run, became `swap` with an empty `HashMap`.
- `yac1p1`: the sorted lists are indexed with `Int` and `.last!` instead of `Rank`.

## Still broken: an old API, but the 0.2.2 rewrite hits a real gap

These ports use an API that was removed or renamed, but rewriting them to the 0.2.2 form runs into a missing feature or a compiler bug.

- `ccc03s4`, `ccc20s3`: the old `StrHash` becomes `RollingHash.new(s)`, but on a `[domain S]Char` from `read()`, `hash.get(l..=r)` or `hash.slice(..)` fails inside the stdlib generic with "a `for` can't walk over an `Int`", reported at 1:1.
- `ccc23s4`: the old `dijkstra(g, from)` becomes `dijkstra(g, [(from, 0)], a => a.1)`. On a growing `[V]List<(V, Int)>` adjacency this fails with "indexed by `V`, but the index is a `(V, Int)`" at 1:1, because a tuple adjacency has no `.to` arc.
- `ceoi09p3`: `let (H, height) = compress(ys)` becomes `let height = compress(ys)`, but `height.D` is "not a type `cpl run` knows" in a fn signature (`-> height.D`). `domain H = height.D` makes a separate domain that doesn't match `height.D`.
- `cco18p1`: `read(columns)` is a designed but unbuilt library feature (decisions.md), not a removed one.

## Still broken: an old API, fix not landed

- `noi16p4`: `let coord = compress(ends)` with `LazySegTree<coord.D, Int>(AddMax)`, `.apply(range, times)` and `.all` checks and builds. Its output differs from `noi16p4_orig.cpp` on random inputs, and the original prints nothing on some of them, so the old `add(l, r)` bounds have not been confirmed. Left as it was.
- `nccc10s2`: `Perm<L>` was removed. A `[n, k]L` read with pairwise loops checks and builds, but matches the original on only 10 of 40 inputs. That's probably the input format, since the original reads one string per order. Left as it was.
- `ioi05p6`: `WithMin<Village>` was removed, and the port also has a `read(gen:) where` guard. It needs a redesign of its sentinel domain. Not attempted.
- `ccc21s4`: `where` becomes `assert` and the port checks clean, but the emitted C++ still fails to compile (see below), so it stays in `PENDING`.

### Real gaps or compiler bugs

These still fail when written the 0.2.2 way.

- `ccc21s4`: **codegen bug.** Once its `where` guard becomes `assert`, it passes `cpl check` and `cpl build`, but the emitted C++ does not compile. The errors are "cannot convert `std::optional<long long>` to `ll` in return" and `operator!=` on `std::optional<std::function<...>>`, plus a missing `cpl_tie` on that optional. The checker accepts something the C++ backend cannot lower: a function-typed optional that is compared and destructured.
- `dpr`: **`Mod<_>` inference.** The error is "expected `[V, V]Mod<_>`, found `[V, V]Mod<1000000007>`" (19:31), on `walks = mul(walks, step)` with `type Mat = [V, V]Z`. The same failure appears in `scout_batch_11/noi21p2`.
- `apio07p1`: `where` guards were removed (34:48). Behind it, `read()` has no input format for a `?Bar` (33:23), so optional input is not readable yet.
- `ccc21s2`: `where` guards were removed (5:28). Behind it, `read()` has no input format for a `Stroke` (15:26). Enums cannot be read yet.
- `noi02p1`: `read()` has no input format for an `Op` (23:22). Enums cannot be read yet.
- `mccc4j5`: `read()` has no input format for a `List<V>` (4:48).
- `coci20c6p3`: `read()` can't tell how many values a `[W]Letter` has (14:34). This is a word read into a fresh domain, `let word: [domain W]Letter = read()`.
- `noi01p5`: no `product` in scope (11:5).

The read gaps (`?T`, enums, `List<V>`, a fresh domain from a token) are the same family as "read() for every type that can be read" in `research/friction-queue.md`. `product` is in the Numbers cluster there.
