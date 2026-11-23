# Friction Backlog — Stage-1 Scout Batch 19 (Interactive, Custom Checkers, Generators, and Unusual I/O)

Batch 19 ported 10 previously unported problems from `corpus/` (verified zero overlap against all prior ports, `scout_batch_1`–`scout_batch_18`, `blind_critic_1`, and `blind_critic_2`) chosen specifically to stress CP Lang's testing and I/O boundaries: 4 interactive problems spanning stdio and IOI grader protocols (`egoi23p3`, `ioi07p1`, `ioi16p5`, `ioi14p3`), 3 multi-answer constructive problems requiring custom checkers (`ccc23s3`, `dmopc21c5p4`, `coci21c6p3`), and 3 unusual-I/O problems covering sentinel-terminated multi-test streams, mixed cell alphabets, recursive grammar strings, and verbatim line-oriented parsing (`ccc04s5`, `ceoi22p2`, `noimps22p1`).

All 10 ideal sketches live in `proto/ports/scout_batch_19/`, and 9 of 10 have verified buildable ports in `proto/ports/scout_batch_19/today/` compiled with `proto/cp.py build` + `g++ -std=c++17 -O2 -fsanitize=address,undefined` and checked against their original C++ submissions and independent checkers/interactors via `python3 proto/ports/scout_batch_19/verify.py`. The 10th (`noimps22p1`) is hard-blocked on `proto/cpc` lacking `read_line()` and mutable/sliceable `Str` operations.

Across the 10 problems, original C++ totals 1,306 raw LOC and 981 template/fast-IO-stripped LOC. Ideal CP Lang solutions total 312 LOC (76.1% raw / 68.2% stripped reduction) plus 189 LOC of co-located `interactor:`, `checker:`, `gen`, and `spec` blocks (18.9 LOC per problem on average). Even without those features, the 9 buildable `today/` ports total 469 LOC vs 1,200 raw / 899 stripped C++ (60.9% raw / 47.8% stripped reduction).

---

## Architectural Synthesis: Unifying `interactor`, `checker`, `ask`, `gen`, and Unusual I/O

Porting all 10 problems side by side revealed a major simplification opportunity for Stage 2 (Language Architect) and Stage 3 (Subtraction Auditor). Today, `plan/15-program-structure.md`, `plan/16-io.md`, and `plan/18-key-features.md` sketch `protocol`, `@interactor`, `@checker`, `ask(...)`, `gen`, `where`, and `spec` as separate concepts with different syntax and conventions. In practice, every single one of these 10 problems fits inside **three small, orthogonal mechanisms**:

1. **One harness anatomy for both `checker:` and `interactor:`.**
   A `checker:` block is simply a zero-query `interactor:` block. Both live at the bottom of the `.cp` file, both inherit the solution's top-level input bindings (`n`, `m`, `r`, `c`, `edges`, `ns`), both read the solution's stdout with the exact same `read_output(...)` primitives as `read(...)`, both assert properties with `require`, and both are stripped completely by `cpl build` with zero emitted-C++ cost.

2. **Drop standalone `protocol` declarations in favor of direct `on ask(...)` and `extern`/`export` binding inside `interactor:`.**
   In `plan/16-io.md`, writing an interactive judge requires declaring a `protocol` block, then an `@interactor(Protocol)` struct implementing it, then a `@graded` solution taking `judge: Protocol`. On real stdio problems (`egoi23p3`, `ioi07p1`), contestants already write `ask("?", moves)` and `ask("examine", x, y)`. Repeating the query signature in a third `protocol` block is pure ceremony. Instead, `interactor:` matches `on ask("?", moves: Str) -> Cell:` directly for stdio problems, `on add_element(x: Str):` for IOI `extern fn` graders (`ioi16p5`), and `run(sol: Game):` for reversed-control callback graders (`ioi14p3`).

3. **Separate dense constraints (`assume`) from sparse/grammatical/coupled invariants (constructive `gen` with parameter shrinking).**
   Putting `assume` next to `read()` works cleanly for dense invariants like `assume h * w >= 2` (`egoi23p3`) or `assume box != (0, 0)`. It completely fails whenever random inputs have near-zero validity probability: a random `(x0, y0)` lands on the hidden 5x5 checkerboard in `ioi07p1` with probability `~1/n^2`; a random wall grid in `ccc04s5` disconnects the start from the finish; a random string is never a valid `min`/`max`/`?` expression tree in `ceoi22p2`; and random preprocessor lines in `noimps22p1` crash on `#undef` of undefined macros. Constructive `gen` blocks solve all four, and critically, `cpl stress` should shrink the **generator's parameters** (`m`, `leaves`, `rows`, `cols`, `n`) rather than mutating serialized bytes and breaking input validity.

4. **Keep unusual I/O out of the core grammar via three small I/O primitives.**
   Across `ccc23s3`, `dmopc21c5p4`, `coci21c6p3`, `ccc04s5`, `ceoi22p2`, and `noimps22p1`, we never needed regex literals or a parser generator in the language core. Almost all unusual-I/O friction vanishes with three orthogonal additions:
   - symmetric `none:` sentinels on `read(none: ...)` / `print(ans, none: "IMPOSSIBLE")` / `read_output(none: -1)`,
   - sentinel-terminated stream loops (`for (m, n) in read_until((0, 0)):`) plus char-range `@tags('.', '*', '1'..='9')` enums, and
   - `read_line()` + a real byte-buffer `Str` (`push`, `repeat`, slicing, `starts_with`, `split_once`, `parse_int`).

---

## Numbered Friction Findings

### B19-1: `ask(...)` lacks inline query limits, custom `Bool` reply tokens, and expression form

**Problems:** `egoi23p3`, `ioi07p1`.

In `proto/cpc` today, `ask(...)` requires a single named binding (`let got: (Row, Col) = ask("?", moves)`), accepts no `limit:` clause, and has no way to read `"true"`/`"false"` directly as `Bool`. In `ioi07p1`, reading the reply as `Str` and comparing `rep == "true"` works, but loses static checking that the judge only sends `"true"` or `"false"`, and the query budget (`<= 2` in `egoi23p3`, `<= 300` in `ioi07p1`) lives only in comments or an external Python script.

**Before (today's `proto/cpc`):**
```cp
fn examine(x: Int, y: Int) -> Bool:
    if x < 1 or x > n or y < 1 or y > n:
        return false
    let rep: Str = ask("examine", x, y)
    rep == "true"

// In egoi23p3: cannot state the 2-query limit next to ask()
let got: (Row, Col) = ask("?", moves)
```

**After (unified CP Lang):**
```cp
fn examine(x: Int, y: Int) -> Bool:
    if x < 1 or x > n or y < 1 or y > n:
        return false
    ask("examine", x, y, limit: 300) -> Bool(true: "true", false: "false")

// In egoi23p3:
let got: Cell = ask("?", moves, limit: 2)
```

Allowing `ask(...)` as an expression (returning the parsed reply when its type is known from `-> T` or the enclosing function return type) and supporting `limit: K` directly on `ask(...)` gives immediate local query-limit traps even before writing an `interactor:` block, and cross-checks against the `interactor:`'s `limit` declaration during `cpl stress`.

---

### B19-2: Stdio `interactor:` needs co-located hidden state, `on ask(...)` dispatch, and automatic shrinking

**Problems:** `egoi23p3`, `ioi07p1`.

Today, verifying `egoi23p3` or `ioi07p1` requires writing an external Python `Judge` class (35–50 lines of subprocess pipes, string splitting, and manual `random.Random` loops, as seen in `proto/interact.py` and `verify.py`) that cannot share the solution's `simu()` helper or domain types (`Row`, `Col`, `Cell`). When a bug triggers on a 15x15 grid, there is zero automatic shrinking to a 2x2 counterexample.

**Before (external Python harness in `verify.py`):**
```python
p = subprocess.Popen([exe], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, bufsize=1)
p.stdin.write(f"{h} {w}\n"); p.stdin.flush()
for line in p.stdout:
    parts = line.strip().split()
    if parts[0] == "?":
        qs += 1; assert qs <= 2
        ...
```

**After (co-located `interactor:` in `egoi23p3.cp`):**
```cp
interactor:
    hidden box: Cell = gen()
    assume box != (Row.first, Col.first)
    limit queries <= 2

    on ask("?", moves: Str) -> Cell:
        require moves.len <= 20000
        require all(ch in "<>^v" for ch in moves)
        simu(box, moves)

    check:
        let (tag, r, c): (Str, Row, Col) = read_output()
        require tag == "!" and (r, c) == box
```

Because `interactor:` shares the solution's scope, `egoi23p3`'s interactor reuses `simu(box, moves)` in one line instead of duplicating the grid-wall simulation in Python, and `cpl stress` shrinks `(h, w, box)` jointly while preserving `h * w >= 2` and `box != (0, 0)`.

---

### B19-3: Non-typeable, sparse, grammatical, and cross-test constraints need constructive `gen` + parameter shrinking

**Problems:** `ioi07p1`, `ioi16p5`, `dmopc21c5p4`, `ccc04s5`, `ceoi22p2`, `noimps22p1`.

Six of our ten problems have input invariants that static types cannot express and rejection sampling (`assume`) cannot satisfy:
- `ioi07p1`: `(x0, y0)` must lie inside a red square of a hidden 5x5 checkerboard inside `1..=n` (random hit rate `~1/n^2`).
- `ioi16p5`: `n` must be a power of 2 (`2..=128`), and `w, r` scale with `n * log2(n)`.
- `dmopc21c5p4`: across `t` independent test cases, `sum(ns) <= 1_000_000`.
- `ccc04s5`: the grid must contain at least one obstacle-free path from bottom-left `(rows-1, 0)` to bottom-right `(rows-1, cols-1)`, and the multi-test stream must end with `0 0`.
- `ceoi22p2`: the input string must belong to the context-free grammar `E ::= '?' | "min(" E "," E ")" | "max(" E "," E ")"`.
- `noimps22p1`: `#undef X` is only legal when `X` is currently defined.

**Before (today's `proto/cpc`):**
None of these 6 can use `cpl stress` without an external Python generator; `read(gen: ...)` only covers independent scalar/array ranges. Even if `assume` existed, rejection sampling loops forever on `ioi07p1` and `ceoi22p2`.

**After (constructive `gen` + `assume`):**
```cp
// Dense invariant (egoi23p3) or aggregate budget (dmopc21c5p4):
assume h * w >= 2
assume sum(ns) <= 1_000_000

// Sparse / grammatical invariant (ceoi22p2):
gen s:
    fn build(leaves: Int) -> Str:
        if leaves == 1: return "?"
        let left = rng.int(1..<leaves)
        let op = rng.choice(["min", "max"])
        f"{op}({build(left)},{build(leaves - left)})"
    let leaves = rng.int(1..=min(size, 1_000_000))
    yield build(leaves)
```

Crucially, `cpl stress` should record the random choices made inside `gen` (a Hypothesis/QuickCheck-style choice stream) and shrink those choices directly (`leaves` 1000 -> 5 -> 3; `m` 25 -> 1; `rows`/`cols` -> 2x2). Shrinking the choice stream guarantees every shrunk test case remains 100% grammatically and structurally valid without writing a separate `shrink` function.

---

### B19-4: Two-phase IOI grader APIs need `extern fn` / `export fn` with per-method limits and phase ordering

**Problems:** `ioi16p5`.

IOI grader problems (`messy.h` in `ioi16p5`) do not use `stdin`/`stdout` at all on the judge: the grader links against `restore_permutation(n, w, r)` and provides `add_element(s)`, `compile_set()`, and `check_element(s)` with **separate** call limits (`<= w` writes, `== 1` compile, `<= r` reads) and a strict phase barrier (`add_element` forbidden after `compile_set`, `check_element` forbidden before it). In `today/ioi16p5.cp`, we had to invent a fake `"A"`/`"C"`/`"Q"`/`"!"` stdio protocol just to test the algorithm in `proto/cpc`.

**Before (`today/ioi16p5.cp` fake stdio shim):**
```cp
fn add_elem(s: [Pos]Char):
    let ack: Int = ask("A", s)
    assert ack == 1
fn compile_all():
    let ack: Int = ask("C")
    assert ack == 1
fn check_elem(s: [Pos]Char) -> Bool:
    let rep: 0..=1 = ask("Q", s)
    rep == 1
```

**After (direct `extern fn` / `export fn` + `interactor:`):**
```cp
extern fn add_element(x: Str)
extern fn compile_set()
extern fn check_element(x: Str) -> Bool

export fn restore_permutation(n: 2..=128, w: Int, r: Int) -> List<Int>:
    ...

interactor:
    limit add_element <= w
    limit check_element <= r
    limit compile_set == 1
    on add_element(x: Str):
        require not compiled
        raw.insert(x)
    on compile_set():
        require not compiled
        compiled = true
        ...
    on check_element(x: Str) -> Bool:
        require compiled
        x in scrambled
    check(got: List<Int>):
        require compiled and got == perm
```

With `cpl build`, `extern fn` emits `#include` / forward declarations matching the contest header and `export fn` emits the C++ entrypoint (with no `main()`), while `interactor:` is stripped. Locally, `cpl stress` compiles `interactor:` into `main()` and links it directly to `extern`/`export` — zero subprocess pipes, zero serialization overhead, and full ASan/UBSan + shrinking.

---

### B19-5: Reversed-control adaptive graders need `export grader` and driver-mode `interactor: run(sol)`

**Problems:** `ioi14p3`.

`ioi14p3` (IOI '14 Game) flips the interactive polarity completely: the **judge** is in control and invokes the solution's `initialize(n)` once followed by `has_edge(u, v)` `n*(n-1)/2` times, checking an invariant after **every single call** (the revealed edges must remain disconnected until the very last query, and form a spanning tree at the end). Neither `ask(...)` nor passive `on fn(...)` handlers can express a judge that drives the solution across stateful callbacks.

**Before (`today/ioi14p3.cp` inverted stdio loop):**
```cp
let total_pairs = n * (n - 1) / 2
var reply = 0
for step_i in 0..total_pairs:
    if step_i == 0:
        let e: (V, V) = ask("READY")
        reply = has_edge(mut par, mut comp_sz, mut edge_cnt, e.0, e.1)
    else:
        let e: (V, V) = ask(reply)
        reply = has_edge(mut par, mut comp_sz, mut edge_cnt, e.0, e.1)
print(reply)
```

**After (`export grader` + driver-mode `interactor: run(sol)`):**
```cp
export grader Game:
    domain V = 0..n
    var dsu = Dsu(V)
    var asked: [V, V]Int = 0
    fn initialize(n: 2..=1500): pass
    fn has_edge(u0: V, v0: V) -> 0..=1: ...

interactor:
    gen (n, order): ...
    run(sol: Game):
        sol.initialize(n)
        var yes = Dsu(0..n)
        for (idx, (u, v)) in order.enumerate():
            let ans = sol.has_edge(u, v)
            if ans == 1:
                require yes.find(u) != yes.find(v)
                yes.merge(u, v)
            if idx + 1 < order.len:
                require yes.components > 1
        require yes.components == 1
```

`export grader` also scopes persistent cross-call state (`dsu`, `asked`) to an instance instead of mutable C++ globals, letting `cpl stress` run thousands of fresh `Game` trials in one process with zero state leakage between trials.

---

### B19-6: Symmetric `none:` sentinels on `read`/`print` and char-range `@tags` for mixed cell alphabets

**Problems:** `ccc23s3`, `dmopc21c5p4`, `coci21c6p3`, `ccc04s5`.

Four separate problems hit sentinel or mixed-symbol token I/O:
- `ccc23s3` prints either a `[Row, Col]Char` grid or `"IMPOSSIBLE"`, whereas `proto/cpc`'s `print(?T)` hardcodes `-1` for `none`.
- `dmopc21c5p4` and `coci21c6p3` print either a valid construction or `-1`, and their checkers must read back that exact sentinel cleanly.
- `ccc04s5` reads a grid whose cells are `'.'` (empty), `'*'` (wall), or `'1'..'9'` (coin value). Current `@tags` on enums supports single literals, not a digit payload range `'1'..='9'`, forcing manual `if ch >= '0' and ch <= '9': gain += ch - '0'` ASCII math.

**Before (`today/ccc23s3.cp` & `today/ccc04s5.cp`):**
```cp
// ccc23s3: cannot print ?[Row, Col]Char with custom sentinel
if not ok:
    print("IMPOSSIBLE")
else:
    print(grid)

// ccc04s5: manual ASCII parsing on raw Char grid
let ch = grid[R(r), C(c)]
if ch == '*': break
if ch >= '0' and ch <= '9': gain += ch - '0'
```

**After (unified CP Lang):**
```cp
// Symmetric `none:` sentinel across print / read / read_output:
print(construct(n, m, r, c), none: "IMPOSSIBLE")
let out: ?[Row, Col]Char = read_output(none: "IMPOSSIBLE")

// Char-range payload tag for mixed cell alphabets (ccc04s5):
@tags('.', '*', '1'..='9')
enum Cell: Empty | Wall | Coin(val: 1..=9)
```

One optional `none:` parameter on `read`, `read_output`, and `print`, plus char-range tags on `@tags`, handles both sentinel outputs/inputs and mixed ASCII grids cleanly.

---

### B19-7: Co-located `checker:` needs `read_output()` for variable-size outputs, multi-test loops, and impossibility cross-checks

**Problems:** `ccc23s3`, `dmopc21c5p4`, `coci21c6p3`.

A simple `@checker(output: T) fn valid(out: T) -> Bool` signature (as sketched in `plan/15-program-structure.md`) breaks on three real patterns we hit immediately in Batch 19:
1. **Variable-size outputs (`coci21c6p3`)**: the solution outputs either `-1` or an integer `k` (`0 <= k <= 200_000`) followed by `k` operation pairs `(u: V, d: 0..=1)`. The output length is chosen by the solution itself!
2. **Multi-test outputs (`dmopc21c5p4`)**: `t` test cases produce `t` heterogeneous outputs (some `-1`, others length-`ns[c]` permutations), which are awkward to package into a single static return type.
3. **False `IMPOSSIBLE` / `-1` detection (`ccc23s3`, `dmopc21c5p4`, `coci21c6p3`)**: a checker that only validates constructions when `out != none` passes a buggy solution that prints `"IMPOSSIBLE"` or `-1` on every input! The checker must cross-check `none` against a feasibility predicate or a small-input brute-force `spec`.

**Before (imperative `grade()` baked into `5350806.cpp` or external Python in `verify.py`):**
Ryan embedded a 28-line `void grade(...)` directly inside his `ccc23s3` C++ submission (`5350806.cpp:127-154`) and ran it at runtime on the judge — risking TLE/WA if `grade` had a bug, and still unable to catch a false `"IMPOSSIBLE"`.

**After (co-located `checker:` with `read_output()` in `coci21c6p3.cp` and `ccc23s3.cp`):**
```cp
// Variable-length op sequence + simulator + feasibility cross-check (coci21c6p3):
checker:
    let k: -1..=200_000 = read_output()
    if k == -1:
        require not is_peelable(V, edges), "printed -1 on a solvable graph"
    else:
        let ops: [0..k](V, 0..=1) = read_output()
        var last_t: [V]Int = 0
        var last_d: [V]0..=1 = 0
        for (t, (u, d)) in ops.enumerate():
            last_t[u] = t + 1; last_d[u] = d
        for (u, v) in edges:
            require max(last_t[u], last_t[v]) > 0
            require (if last_t[u] > last_t[v]: last_d[u] == 1 else: last_d[v] == 0)
```

Making `checker:` read via `read_output()` (reusing the same dependent-domain `read()` machinery already in the language) handles fixed grids, dependent-length sequences, and multi-test loops uniformly, and strips out completely on `cpl build`.

---

### B19-8: Two live compiler-trust bugs hit during Batch 19 (`std::extent` collision & `assert` side-effect erasure)

**Problems:** `ioi07p1` (and reproduced in `proto/cpc` during Batch 19 prototyping).

Both bugs directly violate the P0 "Zero-UB compiler trust / zero compiler fights" goals:

1. **C++ `std::` namespace leakage (`ioi07p1`):**
   Writing `fn extent(dx: Int, dy: Int) -> Int` compiled through `proto/cpc` cleanly, then failed in `g++` with `error: reference to 'extent' is ambiguous` against `std::extent` from `<type_traits>` because emitted C++ uses `using namespace std;`. We had to rename `extent` to `reach` in `today/ioi07p1.cp`.
   - **Fix:** Stop emitting `using namespace std;` in generated C++ (qualify `std::` explicitly in the tiny runtime header) or emit user code inside `namespace cp_user {}`. Eliminating `using namespace std;` keeps generated C++ clean, readable, and 100% immune to standard-library name clashes (`extent`, `rank`, `data`, `size`, `next`, `prev`, `count`, `left`, `right`, `move`, `distance`).

2. **Silent side-effect erasure in `assert` (reproduced in `proto/cpc`):**
   Writing `assert solve_grid(mut g)` compiles with zero warnings, emits `CP_ASSERT(solve_grid(g), ...)`, and under `g++ -O2` (`CP_ASSERT` compiled to a no-op!), `solve_grid` is **never executed**!
   - **Fix:** Statically reject `mut` arguments or side-effectful calls inside `assert` (`error: assert condition mutates 'g'; bind the result first or use require`), catching C++'s classic `NDEBUG` trap at compile time.

---

### B19-9: Verbatim line-oriented I/O (`read_line()`), sentinel-terminated streams (`read_until`), and byte-buffer `Str`

**Problems:** `noimps22p1` (hard blocker), `ccc04s5`, `egoi23p3`, `ioi16p5`.

`proto/cpc` has no `read_line()` / `getline`, no sentinel-terminated multi-test loop helper, and almost no methods on `Str` (`Str` lacks `.push(ch)`, slicing `s[a..b]`, `.starts_with(...)`, `.split_once(...)`, `.parse_int()`, and `"v>^" * k` repetition). That caused four concrete failures:
- `noimps22p1` could not be built in `today/` at all: `FastIn` skips all leading/trailing spaces and blank lines, destroying the input of a C preprocessor that must preserve whitespace verbatim and emit empty lines for `#define`/`#undef`.
- `ccc04s5` had to write `while true: let m = read(); let n = read(); if m == 0 and n == 0: break; solve_case(m, n)` instead of a clean sentinel loop.
- `egoi23p3` had to write an `append_rep(mut out: List<Char>, pat: List<Char>, times: Int)` helper and carry `List<Char>` everywhere instead of `Str` (`"v>^" * (w - 1)`).
- `ioi16p5` hit `Str has no method 'push'` and had to fall back to `[Pos]Char`.

**Before (`today/egoi23p3.cp` & `today/ccc04s5.cp`):**
```cp
// egoi23p3: manual List<Char> builder because Str has no push/+/*
fn append_rep(mut out: List<Char>, pat: List<Char>, times: Int):
    for _ in 0..times:
        for ch in pat:
            out.push(ch)

// ccc04s5: manual infinite loop + sentinel break
while true:
    let m: 0..=100 = read()
    let n: 0..=100 = read()
    if m == 0 and n == 0: break
    solve_case(m, n)
```

**After (unified byte-buffer `Str` + `read_line()` + `read_until()`):**
```cp
// egoi23p3:
fn sweep_moves() -> Str:
    var out = ""
    for i in 1..h:
        out += if i % 2 == 1: "v>^" * (w - 1) + ">v>" else: "v<^" * (w - 1) + "<v<"
    out

// ccc04s5:
for (rows, cols) in read_until((0, 0)):
    ...

// noimps22p1:
let n: 1..=100 = read_line().parse_int()!
let lines: [0..n]Str = [read_line() for _ in 0..n]
if line.starts_with("#define "):
    let rest = line[8..].trim_start(' ')
    let (name, body) = rest.split_once(' ') ?? (rest, "")
```

Because CP Lang commits to ASCII-only byte strings (`Char` = 1 byte, zero UTF-8), all of `Str.push`, `Str + Str`, `Str * Int`, `s[a..b]`, `starts_with`, `trim_start`, `split_once`, and `parse_int` are trivial userland methods on the flat byte array, and `read_line()` only needs to share the buffered input reader's byte cursor.

---

### B19-10: Grid transpose views and reverse/directional sweeps

**Problems:** `ccc23s3`, `ccc04s5`.

Two grid problems hit mechanical grid-ergonomics friction in `today/`:
- In `ccc23s3`, symmetry between `r == 0` / `c == 0` and `r == n` / `c == m` is naturally expressed by transposing the grid (just as `5350806.cpp` calls `transposed(sn(m, n, c, r))`). In `today/ccc23s3.cp`, because `[Row, Col]Char` is tied to the outer `Row`/`Col` domains and cannot be transposed cleanly, we had to thread a `flip: Bool` flag and a `put(mut g, flip, i, j, ch)` indirection through every helper.
- In `ccc04s5`, sweeping downward (`start..rows`) and upward (`start` down to `0`) over the same column duplicated the entire inner loop body (`12` lines copied verbatim) because `proto/cpc` lacks a reversible range value (`[start..R.end, (R.first..=start).rev]`) and `while c >= 0` had to replace `for c in C.rev`.

**Before (`today/ccc23s3.cp`):**
```cp
fn put(mut g: [Row, Col]Char, flip: Bool, i: Int, j: Int, ch: Char):
    if flip: g[Row(j), Col(i)] = ch
    else: g[Row(i), Col(j)] = ch
```

**After (domain-indexed `.transposed()` view):**
```cp
if c == 0:
    return construct(m, n, c, r)?.transposed()
```

---

### B19-11: Subtraction proposal for Stage 3 — delete standalone `protocol` from the core language

**Problems:** `egoi23p3`, `ioi07p1`, `ioi16p5`, `ioi14p3`.

`plan/16-io.md` currently introduces three separate constructs for interactive problems: `protocol`, `@interactor(P)`, and `@graded fn ... (judge: P)`. Across all four interactive archetypes in Batch 19 (stdio 2-query, stdio 300-query, two-phase IOI grader, reversed-control adaptive IOI grader), **zero problems benefited from a standalone `protocol` interface**. Every contest problem has exactly one judge protocol!

Deleting standalone `protocol` and unifying everything under:
- `ask(..., limit: K)` for stdio queries,
- `extern fn` / `export fn` / `export grader` for IOI C++ grader ABIs,
- `interactor:` (with `hidden`, `gen`, `limit`, `on ...`, `run(...)`, `check:`) for local testing/shrinking, and
- `checker:` (as the 0-query subset of `interactor:` with `read_output()` and `require`)

cuts an entire top-level keyword and trait-dispatch layer from the one-page cheat sheet while actually covering **more** contest formats (including reversed-control `ioi14p3`, which `protocol` + `@graded` could not express cleanly).
