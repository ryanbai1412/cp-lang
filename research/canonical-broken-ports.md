# Canonical ports that fail on HEAD

These are the canonical `proto/ports/*.cp` files (outside `ecnerwala/`, `scout_batch_*` and the critic folders) that had stale-port lint hits but failed `cpl check` or `cpl build` on HEAD at the start of stale-port cleanup Batch #12. There were 22 of them. Each entry gives the first compiler error, reproduced with `target/release/cpl check proto/ports/NAME.cp`.

Eight of them use a removed `where` guard. Replacing it with `assert` is all that `noip06p2` and `yac6p3` need: with that change both check, build and compile, and `noip06p2` matches its original C++ on 40/40 random cases. The fix is not applied yet, because both files are listed in `PENDING`/`NOT_BUILT` in `crates/cpl-cli/tests/corpus.rs`, and that test fails once they check clean. Landing them needs the port edit and the removal of those two lines from `corpus.rs` in the same commit. In the other six, converting the guard just exposes a second error, so their entry shows that second error.

## Pre-0.2.2 syntax or a renamed API: port cleanup only

Each of these ports was written against an older surface, and the error goes away by rewriting it to the 0.2.2 form. None of them needs a compiler change.

- `noip06p2`: `where` guards were removed (10:75). `assert` fixes it; see above.
- `yac6p3`: `where` guards were removed (8:25). `assert` fixes it; see above.
- `dpy`: `where` guards were removed (14:29). Behind it, `no distinct in scope` (14:8); the 0.2.2 form is `.deduped()`.
- `cco16p2`: `where` guards were removed (21:44). Behind it, `sort` takes nothing or a key (34:5); the 0.2.2 form is `xs.sort(by: x => x.w)`.
- `noi16p4`: `where` guards were removed (14:40). Behind it, `AddMax<X>` takes 4 arguments (id, compose, apply, over) (23:13); the 0.2.2 form is a named `Action(id:, compose:, apply:, over:)`.
- `ioi05p6`: no type or domain named `WithMin` (8:13).
- `cco18p1`: `read` takes 0 arguments, but 1 was given (14:21). The 0.2.2 form is a typed `read()` into `[n]T`.
- `ceoi09p3`: a `Compressed<D_1, Int>` can't be taken apart into 2 names (12:5).
- `ioi11p2`: `Rooted` has no parameter named `weight` (13:38).
- `ccc03s4`: no `StrHash` in scope (7:16).
- `ccc20s3`: no `StrHash` in scope (12:12).
- `ccc23s4`: `dijkstra` takes 3 arguments (next, from, w), but `w` is missing (25:9).
- `nccc10s2`: no type or domain named `Perm` (5:16).
- `yac1p1`: expected `Int`, found `Rank` (15:11). Here `sorted(a_in)` returns a plain `List`, and the port still indexes it with `Rank.last`.

## Real gaps or compiler bugs: need a compiler or stdlib change

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
