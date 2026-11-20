# Max-N Judge & Latency Audit #2: the Rust `cpl`

Audit #1 ([max-n-judge-audit-1.md](max-n-judge-audit-1.md)) measured the Python
prototype. This one measures the real thing: `target/release/cpl` at `35b13eb`
(`cargo build --release -p cpl-cli`), g++ 11 with `-std=c++17 -O2`, 8 cores.

Reproduce both halves:

```sh
cargo build --release -p cpl-cli
python3 bench/rust_latency_audit.py            # toolchain budgets, about 2 min
python3 bench/rust_max_n_audit.py --runs 5     # 40 ports at judge limits, about 7 min
```

## Verdict

The front end is far inside every latency budget and the emitted files are
small. Where the emitted C++ is correct it runs at a median 1.15x of the
accepted original. But four of the 40 ports fail at the judge limit, and three
of those four are the compiler's fault: **`@memo` is not implemented** (the
decorator is accepted and ignored by both the interpreter and the back end),
and `[domain S]Char = read()` reads nothing. Those two are the blockers; the
constant-factor list further down is second priority. `cpl stress` misses its
first-case budget and cannot generate inputs for most real ports.

## 1. Toolchain latency

Fresh process per measurement, spawn to exit, median of 7, over the 40 audited
ports (22 to 219 lines of `.cp`). "Worst" is the slowest file.

- **`cpl check` (budget 50 ms): pass.** Median 1.8 ms, worst 22.5 ms.
- **`cpl lsp`, warm (budget 100 ms): pass.** An incremental edit to
  `publishDiagnostics`: median 0.19 ms, worst 7 ms. Hover round trip: median
  0.04 ms, worst 1.4 ms.
- **`cpl run`, debug interpreter (budget 200 ms): pass.** The 17 CLI golden
  programs with their inputs: median 1.8 ms, worst 7.3 ms. That is startup; see
  the throughput note below.
- **`cpl build` (budget 1 s, 64 KB): pass.** Median 2.4 ms, worst 26 ms.
  Emitted files: median 9.4 KB, largest 22 KB (`noi22p5`).
- **`cpl stress`, first case (budget 1 s): fail.** On the 21 ports where a case
  actually ran, cold time is median 1.73 s, worst 2.69 s; all 21 are over. The
  time is the two `g++ -O2` builds (solution and brute), which run one after
  the other. With the binary cache warm the same command takes a median 82 ms.

The worst `check` and `build` files are the ones importing `std.tree` and
`std.graph` (`dpv`, `noip19p2`): 22 to 26 ms against 2 ms for a file with no
imports, so the cost is in loading and checking the imported stdlib modules on
every invocation. That is fine today and is the first thing to watch as the
stdlib grows.

Two things the budgets do not capture:

- **Interpreter throughput.** `cpl run` on medium inputs has a median of 65 ms,
  but 9 MB to 19 MB graph inputs take about 3 s (`apio09p3`, `ioi11p4`), and
  the two `@memo` ports do not finish in 30 s.
- **`cpl stress` coverage.** Of 36 ports tried with the original C++ as the
  brute, 7 agree on a case, 15 stop at "can't generate a ... yet" (named-field
  tuples such as `(u: V, v: V)`, `Str`, an array sized by an earlier value),
  and 13 feed the brute an input it crashes on, because the port reads plain
  `Int` and the generator then produces values like `-999999995` for an `n`.
  The second group is a port-quality problem (untyped reads) that stress makes
  visible; the first is missing generator support.

## 2. Max-N correctness

40 ports: the six Blind Critic #5 Olympiad ports plus DP, graph, tree, grid,
string, range and math ports, each with a worst-case generator at the statement's
limits. Each emitted file is also built with `-fsanitize=address,undefined` and
run on a medium input. **36 of 40 match the original at max-N and are sanitizer
clean**, including all six Blind Critic #5 ports. The four failures:

**`@memo` is a no-op (`dpl`, `ccc18s4`).** `cpl build` emits the plain
recursion:

```cpp
ll trees(ll w) {                 // source: @memo fn trees(w: 1..=1e9) -> Int
    if (w == 1) return 1;
    ...
        total += (last - k + 1) * trees(q);
```

`dpl` (N = 3000) and `ccc18s4` (N = 1e9) time out at 60 s where the originals
take 0.04 s and 0.15 s. Nothing in `cpl-codegen` or `cpl-interp` mentions
`memo`: the IR lowering lists it among the decorators it accepts, and the
checker reads `@memo(into: dp)` for its type, and that is all. `cpl check` and
`cpl lint` pass these files silently, so a contestant gets an exponential
program with no warning. `noi08p4` and `ioi19p5` also use `@memo` and pass only
because their call trees are small.

**`let s: [domain S]Char = read()` reads zero characters (`noip14p6`).** The
domain's length is never set, so the emitted code is

```cpp
std::vector<cpl::u8> s(S_len);          // S_len is still 0
...
a(t, i) = s[(int)0] == '-' ? ...        // unchecked read of an empty vector
```

The interpreter traps cleanly (`S has no position 0: it is empty`); the built
program is undefined behaviour: UBSan reports a null reference and the `-O2`
binary segfaults. This is also the one case found where the build is less safe
than the interpreter on the same program.

**`min` of an empty comprehension (`ioi04p4`).** `min(a * b, min(... for x in
1..=a / 2), ...)` traps when `a == 1`. The Rust semantics (an empty `min` is an
error) is defensible; the port was written against the prototype and never ran
on the Rust compiler. It would then hit the `@memo` problem anyway.

A related gap seen while writing the generators: `opt!` is emitted as an
unchecked `*opt`. An early generator for `ioi09p5` broke the statement's
guarantee and the built program read through an empty optional (ASan SEGV)
instead of failing with a message.

## 3. Runtime and memory at max-N

Median of 5 runs, the 36 correct ports. Time ratio is emitted / original.

Median **1.15x**. 14 ports are at or under 1.0x; 9 are over 1.5x; 7 are over 2x.
Peak RSS median is 1.0x, with 10 ports over 1.5x. The wins are algorithmic or
allocation wins in the port, not codegen: `shoi16p1` (0.27x time, 224 MB against
the original's 853 MB), `noip10p4` (0.01x), `sdoi17r1p1` (0.32x).

The slow ones, with the cause read out of `perf` and the emitted source:

- **`dpv`, 10.2x (0.030 s to 0.301 s).** `Rooted.new` in `std.tree`. 82% of the
  time is in that one function and in `memmove`. Three things stack up: `adj`
  is passed as `std::function<std::vector<tuple<int,int>>(int)>`; `g.adj(u)` is
  a generator that is materialized into a fresh `__yields` vector per call; and
  the DFS stack holds `(node, that vector, cursor)` tuples and copies the top
  tuple, vector included, on every loop iteration.
- **`saspeed`, 5.9x and 3.1x RSS.** `std.string.suffix_array` is prefix doubling
  with a comparison sort (`std::stable_sort` dominates), on `vector<ll>`. The
  original is linear-time. This is a stdlib algorithm choice, not codegen.
- **`noi04p3`, 4.5x.** The DP table is `vector<optional<ll>>` (16 bytes a cell)
  built by `push_back` in a loop, with no `reserve`, once per bitmask.
- **`btoi19p4`, 4.3x.** `[Paid]?Int` again: every cell read copies an
  `optional<ll>`, re-wraps the sum in a new optional and tests it twice. The
  original is a flat `int` array with a sentinel.
- **`noi09p1` 2.8x, `noip19p4` 2.4x, `dpf` 2.1x and 1.9x RSS.** Everything is
  `long long`: `[R, C]Int` becomes `cpl::Grid<ll, 2>` where the original has
  `int dp[n + 1][m + 1]`, although the declared ranges fit in 32 bits.
- **`bsspc21s4`, 1.7x and 3.45x RSS (185 MB against 53 MB).** A 56-byte segment
  tree `Node` (four `ll`, an `optional<ll>`, two `bool`) held in two vectors,
  merged by value through `Node_merge(const Node&, const Node&)`.
- **`noiwc22p3`, 1.5x.** `eval_code` allocates two `std::vector<char>(5)` per
  call for `[5]Bool` locals, and `cand = filtered` copies a `vector<string>`
  each round; `malloc` and `free` are 16% of the profile.
- **`apio09p3` 2.3x RSS, `ioi11p4` 2.0x RSS, `noip19p2` 1.6x RSS.** Two causes:
  the runtime reads all of stdin into one `std::string` before parsing (a 19 MB
  input costs 19 MB that the original's streaming reader never holds), and CSR
  graphs store `ll` offsets and `tuple<int, int>` arcs.

Smaller things in the emitted text: unused stdlib items are emitted (`dpv`
carries `m3_Word`, `m4_Sieve` and six comparison templates it never uses), and
every file compiles in a median 0.58 s with g++, half the original's 1.13 s
because there is no `bits/stdc++.h`.

## 4. What to fix next, in order

1. **Implement `@memo`** in the interpreter and the back end (a dense table over
   domain-typed keys, a hash map otherwise), and until then make `cpl check`
   reject it. Silent exponential time is the worst outcome available.
2. **Size `[domain S]T = read()` from the token read**, and add a differential
   test for it. Emit a checked unwrap for `!` (one predictable branch).
3. **Run this harness in CI.** 596 of the 971 files under `proto/ports` do not
   pass `cpl build` today and six more emit C++ that g++ rejects
   (`noi20p1`, `dpw`, `cco18p5`, `ioi07p3`, `ioi14p2`, `tsoc15c2p6`); "ported"
   currently means "ran on the prototype".
4. **Narrow integers by declared range.** A value or cell typed `1..=1e9`, or a
   domain position, fits `int`; only arithmetic that can overflow needs `ll`.
   This is the single change that moves the most ports (grids, CSR, DP rows).
5. **Represent `[D]?Int` without `std::optional`** when the range leaves a spare
   value (the approved niche-packed `?V`), and stop re-wrapping optionals that
   were just unwrapped.
6. **Inline generators and closures.** A `for a in g.adj(u)` should be the loop
   over `start[u]..start[u + 1]`, not a call that builds a vector, and stdlib
   functions taking `fn` parameters should be monomorphized over the callee
   rather than taking `std::function`. Then fix `Rooted.new` to keep a cursor
   into the adjacency instead of a copy.
7. **Fixed-size arrays for constant-length locals** (`[5]Bool` to
   `std::array`), `reserve` before a counted `push` loop, and move instead of
   copy on `cand = filtered` when the source is dead.
8. **Stream the input** in fixed blocks instead of holding it whole.
9. **`cpl stress`:** build the solution and the brute in parallel (and with
   `-O1`, or the PCH that `cpl run --release` is meant to use) to get the first
   case under 1 s; generate named-field tuples, `Str` and dependent-length
   arrays.
10. **Replace `suffix_array` with SA-IS** or radix-sorted doubling.

## Method notes and limits

- Originals are the corpus submissions the ports were written from. Three Blind
  Critic #5 originals are graders, driven through the stdin drivers in
  `proto/ports/blind_critic_5/check.py`. Outputs are compared token for token,
  except `noi08p4` (0.011 absolute tolerance), `noi04p3` (feasibility and cost;
  the printed plan may tie), `dpf` (any LCS of the best length) and `btoi19p5`
  (length only).
- `bsspc21s4` has no public constraints in the repo; N = M = 1e9, K = 1e5 is
  inferred from the original's types. `noi22p5` keeps values at most n, because
  the original mis-sizes a tree above that. `noiwc22p3` uses 8869 random
  five-letter words, not the real word list.
- Sanitizer runs use medium inputs, except the five ports that reuse audit #1's
  generators, which only have a max size.
- Not measured: `cpl run --release`, `cpl bench`, cold LSP start on a large
  workspace, and any judge other than local g++ 11 on Linux.
- One machine, one session; timings under about 10 ms are dominated by process
  start and should be read as "equal".
