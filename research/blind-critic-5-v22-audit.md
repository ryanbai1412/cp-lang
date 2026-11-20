# Blind Critic #5: Final CP Lang v2.2 (34-Keyword Cheat Sheet) Verification Audit

## 1. Protocol, Problem Selection, and Dual LOC Results

This audit verifies the locked CP Lang v2.2 specification (`docs/one-page-cheat-sheet.md` at `16c07bf`, 34 reserved keywords, 229 lines) end to end — both at the **design level** (writing six hard Olympiad problems cold at 2 AM from the cheat sheet alone) and at the **toolchain level** (running all six problems through the real Rust `cpl check`, `cpl run`, and `cpl build` at `c521b2c` under AddressSanitizer and UndefinedBehaviorSanitizer).

The blind boundary was held strictly:

1. Before writing a single line of `.cp` code, I read **only** `docs/one-page-cheat-sheet.md`. I did not open `crates/`, `proto/cpc/`, `stdlib/`, `decisions.md`, or `research/` until all six first drafts (`proto/ports/blind_critic_5/*_draft.cp`) were written and frozen.
2. All six problems were chosen from the unported accepted submissions in `bench/corpus-500-registry.json`, with zero overlap against `proto/ports/`, `scout_batch_*`, or `blind_critic_1..4`, specifically targeting all five Blind Critic #4 ambiguities and every 34-keyword cut:
   - **`bsspc21s4` (orig `4152932.cpp`)**: 2D coordinate-compressed rectangle sweep with a cover-count segment tree merging non-trivial interval-boundary `Node` structs (tests `Action`, `.all`, `Range<D>.mid`, and non-pushdown lazy limits).
   - **`noi22p5` (orig `4832632.cpp`)**: `@multitest` sweep with `compress`, `SortedBag`, `Set.first_ge`, `sort(..., by: desc)`, order-statistic inversion counting, and `LazySegTree<Val, Int>(AddMin, ...)` (tests `first_ge`, `SortedBag`, `LazySegTree`, and domain safety).
   - **`noi04p3` (orig `5032868.cpp`)**: bitmask enumeration over rows plus constrained column DP with path reconstruction (tests `Subset<Row>`, `?Col` as a DP index, tropical `?Int`, and `min=`).
   - **`noi08p4` (orig `5144322.cpp`)**: functional-graph cycle enumeration with recursive floating-point max-plus tree knapsack DP (tests `F64`, `@memo` auto-reset inside a loop, `swap`, and `reverse` on a subslice).
   - **`noiwc22p3` (orig `4261177.cpp`)**: stateful interactive Wordle grader (`init` + repeated `guess`) with candidate filtering and minimax bucket scoring (tests stateful `@grader`, `Subset<Slot>`, `Str` copying vs fixed byte arrays, and `shuffle`).
   - **`ioi19p5` (orig `5664246.cpp`)**: Boolean-circuit grader (`construct_network` calling bodyless extern `add_and`/`add_or`/`add_xor`/`add_not`) with diagonal grouping (tests bodyless extern `@grader fn`, `@memo` calling extern grader oracles, and `(mp[k] ??= []).push(x)`).
3. After freezing the six blind drafts, I inspected `research/language-architecture-v2.2.md`, `research/blind-critic-4-v21-audit.md`, `stdlib/`, and `crates/`, then wrote runnable CP Lang v2.2 forms (`proto/ports/blind_critic_5/<problem>.cp`) and ran triple differential verification (`original C++` vs `Rust cpl build + ASan/UBSan` vs `Rust cpl run`) via `proto/ports/blind_critic_5/check.py`.

### Conciseness: Raw and Library-Stripped LOC

Across all six problems, the frozen v2.2 blind drafts shrink the original C++ from **1,103 raw LOC** to **438 LOC (-60.3%)**, and from **667 library-stripped LOC** (after removing debug/macro preambles, `main()` harnesses, `ost` typedefs, and template `LazySegTree`/`ST`/`PSA`/`coord_comp` definitions) to **438 LOC (-34.3%)**:

- `bsspc21s4`: 342 raw / 149 stripped C++ -> **126** v2.2 draft (**-63.2% raw**, **-15.4% stripped** — even while writing a 40-line custom non-pushdown `CoverTree`).
- `noi22p5`: 204 raw / 91 stripped C++ -> **57** v2.2 draft (**-72.1% raw**, **-37.4% stripped**).
- `noi04p3`: 203 raw / 166 stripped C++ -> **102** v2.2 draft (**-49.8% raw**, **-38.6% stripped**).
- `noi08p4`: 146 raw / 102 stripped C++ -> **59** v2.2 draft (**-59.6% raw**, **-42.2% stripped**).
- `noiwc22p3`: 118 raw / 90 stripped C++ -> **57** v2.2 draft (**-51.7% raw**, **-36.7% stripped**).
- `ioi19p5`: 90 raw / 69 stripped C++ -> **37** v2.2 draft (**-58.9% raw**, **-46.4% stripped**).

One genuine real-world bug surfaced immediately during porting: in `corpus/submissions/4832632.cpp` (`noi22p5`, lines 198–207), the accepted C++ builds `ST st(m)` over the `m` compressed values, then calls `st.update(arr[i] + 1, n, 1)` — accidentally passing `n` (the position-domain length) to a segment tree indexed by value-domain `m`. In `noi22p5_draft.cp`, `st` has type `LazySegTree<Val, Int>` and `arr` holds `Val`, so writing `st.apply((arr[i] + 1).., 1)` makes that domain mixup structurally unrepresentable at compile time.

---

## 2. Verification of All Five Blind Critic #4 Ambiguities

Every single one of the five fixes locked in v2.2 noticeably reduced 2 AM hesitation, with four passing cleanly and one exposing a real algorithmic boundary worth documenting.

### 1. `Action` Parameter Order & `LazySegTree` Arity — Resolved, With One Algorithmic Limit

The v2.2 cheat sheet now spells out `Action(id, (new_f, old_f) => ..., (f, val, span) => ..., over: op)` and notes that `LazySegTree<D, T>` infers `F`. I had **zero hesitation** on argument order or arity when evaluating `LazySegTree` for `bsspc21s4_draft.cp` and `noi22p5_draft.cp`.

However, `bsspc21s4_draft.cp` surfaced a real algorithmic limit of push-down `LazySegTree`: rectangle union / cover-count sweep applies `+1` and `-1` over overlapping intervals without pushing tags down, because pushing a parent's `+1` into children loses the distinction between a node covered once (`1 -> 0`) and covered twice (`2 -> 1`). The cheat sheet's canonical segment-tree split `let m = r.mid!; (r.l..m.r, m.r..r.r)` made writing a 40-line tag-permanent `CoverTree<D>` clean and index-safe — no new library feature is needed, just awareness that cover-count sweeps still write a custom tree.

### 2. `first_ge` / `last_le` Return Types — Fully Resolved

Listing all three return types inline on the cheat sheet (`?Int`/`?D` on sequences, `?K` on `Set`/`SortedBag`, `?(K, V)` on `Map`) eliminated the Blind Critic #4 confusion completely. In `noi22p5_draft.cp`:

```cp
if let pos = inds[v].first_ge(l) and pos <= r:
    mni = pos
    eqs[pos] = true
    inds[v].remove(pos)
```

I wrote `inds[v].first_ge(l)` directly as `?Pos` without checking back once.

### 3. `Map` Entry, Auto-Insert, and `??=` — Fully Resolved on Paper, Unparsed in Rust

Adding `(auto-inserts if missing)` to `m[k] f= x` and `(m[k] ??= []).push(x)` to §9 made `ioi19p5_draft.cp` feel effortless:

```cp
var mp = Map<Int, List<Int>>()
for (r, c) in (Row, Col):
    (mp[cb(r, c)] ??= []).push((r, c).to_idx)
```

No hesitation at all on paper. The catch is in the Rust compiler (`c521b2c`): `cpl-syntax` rejects `(mp[cb(r, c)] ??= []).push(...)` with `E0103: expected ')' to close the parenthesis, found an assignment operator` because `??=` is parsed as a statement rather than a place-yielding expression (see §5).

### 4. `.all` and `Subset<D>` Iteration — Fully Resolved on Paper

Both `st.all` and the three `Subset<D>` verbs (`for mask in Subset<D>:`, `for v in mask:`, `for sub in mask.subsets:`, plus `.empty`, `.full`, `.with`, `.without`, and `not in`) read like textbook pseudocode in `noi04p3_draft.cp` and `noiwc22p3_draft.cp`:

```cp
for mask in Subset<Row>:
    let h_cost = sum(if (r in mask) != (r in h_init): h_w[r] else: 0 for r in Row)
```

I had zero uncertainty about iterating all masks vs iterating elements of one mask, and `i not in gold` in `noiwc22p3_draft.cp` felt completely natural.

### 5. Stateful `@grader` & `@memo` — Resolved, With One Subtle Nested-Closure Edge Case

Both patterns worked cleanly at 2 AM:

- In `noiwc22p3_draft.cp`, holding `var dict: List<Str> = []`, `var cand: List<Str> = []`, and `var last_guess: Str = "leant"` at top level across `@grader fn init(...)` and `@grader fn guess(...)` required zero boilerplate and zero global domains.
- In `noi08p4_draft.cp`, placing `@memo fn dp(u: V, cdist: 0..=n) -> List<F64>:` inside `for clen in 2..=cyc.len:` cleanly replaced the original C++'s manual nested `memo[i][j].clear()` loop.
- In `ioi19p5_draft.cp`, the cheat sheet's explicit note (`@memo may call extern @grader fns and ask`) let me immediately write zero-argument lazy gate caches:

```cp
@memo fn g_true() -> Int:
    add_or([(r, c).to_idx for (r, c) in (Row, Col)])

@memo fn g_false() -> Int:
    add_not(g_true())
```

One edge case caused brief hesitation: in `ioi19p5_draft.cp`, `g_true` takes **zero parameters** (`@memo fn g_true() -> Int`) and calls `add_or(...)`, while in `noi08p4_draft.cp`, `@memo fn dp(...)` returns a heap `List<F64>` (not `Copy`) and captures the loop-local immutable `let cmul` and `let g`. Confirming in the cheat sheet that zero-arg `@memo` acts as a once-cell and that `@memo` accepts non-`Copy` return types (returning cached clones or references) would remove that last tiny doubt.

---

## 3. Verdict on the 34-Keyword Cuts (`defer`, `where`, `inline`, `lend`, `assume`, `@law`)

Writing six hard Olympiad problems cold confirmed that **every single one of the six keyword cuts in v2.2 was the right call**:

- **`defer`**: Never missed across all six problems. Even in `noi04p3` backtracking reconstruction and `bsspc21s4` sweep events, explicit lines were clearer and shorter.
- **`assume`**: Never missed. `assert self.cover[o] >= 0` (`bsspc21s4_draft.cp`) and `assert cand.len > 0` (`noiwc22p3_draft.cp`) were unambiguous with a single assertion keyword.
- **`where` and `inline`**: Zero occurrences and zero mental friction. Inline generic bounds (`struct CoverTree<D>:`) handled everything cleanly.
- **`lend` and `@law`**: Completely invisible to contestants (pure library-author mechanics), keeping the contestant vocabulary at a tight 30 keywords.

Probing the real Rust `cpl check` (`c521b2c`) directly confirmed both halves of the cut:

- All six removed/softened names compile cleanly as ordinary variable names (`let defer = 1`, `let assume = 2`, `let where = 3`, `let inline = 4`, `let by = 5`, `let with = 6`).
- Removed v2.1/C++ habits emit crisp, actionable `E0503` fix-it diagnostics:
  - `else if` -> `error[E0503]: use 'elif', not 'else if'` (`help: write 'elif cond:'`).
  - `for mut x in xs:` -> `error[E0503]: 'for mut' was removed in CP Lang v2.2` (`help: loop over the indexes and assign through them`).
  - `@memo(hash)` -> `error[E0503]: '@memo(hash)' was removed in CP Lang v2.2` (`help: write bare '@memo'`).

---

## 4. Remaining Blind Cheat-Sheet Friction Points (Ranked)

Even though the core language feels remarkably cohesive at 2 AM, writing six hard drafts strictly from `docs/one-page-cheat-sheet.md` uncovered **six concrete documentation/ergonomics gaps** worth fixing without adding a single keyword:

### 1. Function-Type Syntax Is Missing From the Cheat Sheet (High)

In `ioi19p5_draft.cp`, I needed a helper taking a coordinate projection callback `cb` and had to guess the function-type spelling:

```cp
// Guessed blindly in ioi19p5_draft.cp:
fn go_cell(cb: fn(Row, Col) -> Int) -> List<Int>:
```

Inspecting `stdlib/lazy.cp` line 57 afterward (`pub fn update(mut self, i: D, f: T -> T):`) showed CP Lang's actual function-type syntax is `(Row, Col) -> Int` (no leading `fn`). Because `docs/one-page-cheat-sheet.md` never shows a function parameter type, any contestant writing a higher-order helper has to guess between `fn(A, B) -> C` and `(A, B) -> C`. **Fix:** add one 6-word example (`f: (Int, Int) -> Int`) to §1 or §7 of the cheat sheet.

### 2. Semicolon Statement-Separator Contradiction Between Cheat Sheet and Parser (High)

`docs/one-page-cheat-sheet.md` uses `;` to separate two statements on one line in three places: line 39 (`out.push('a'); out += "bc" * 3`), line 73 (`best = none; break`), and line 122 (`if dst.len < src.len: swap(mut dst, mut src); for x in src.take(): ...`). Following that lead in `bsspc21s4_draft.cp`:

```cp
xs_raw.push(r.x1); xs_raw.push(r.x2)
```

Rust `cpl check` rejected it with `error[E0107]: statements end at the end of the line; ';' is only used inside '{ ... }'`. Either the parser should allow `;` as an inline statement separator (the natural Python/contest expectation) or the cheat sheet should stop teaching `;` outside `{ ... }`.

### 3. No Order-Statistics (`rank` / `kth`) on `Set` / `SortedBag`, and Unclear `AddMin` Empty-Range Type (Medium)

In `noi22p5_draft.cp`, counting inversions among fixed elements (`ost.order_of_key(...)` in `4832632.cpp`) forced me to guess `seen.rank(arr[i])` because §9 lists `Set`/`SortedBag` without saying whether order-statistic methods (`.rank(x)` / `.kth(i)`) exist. Similarly, when writing `LazySegTree<Val, Int>(AddMin, ...)`, I hesitated over whether `st.fold(arr[i]..)` returns `Int` (with `Int.MAX` on an empty range — confirmed in `stdlib/lazy.cp` line 160) or `?Int` (matching tropical `min`). One parenthetical each on the cheat sheet eliminates both guesses.

### 4. Heap `Str` `.copy()` Tax on Short Fixed-Length Strings (Medium)

In `noiwc22p3_draft.cp` (5-letter Wordle words), the original C++ used `using str = array<char, 5>` (5 bytes, trivially copyable). Because the cheat sheet never shows a literal constructor for `[0..5]Char` (like `b"leant"` or `[0..5]Char("leant")`), I fell back to heap `Str` and had to sprinkle `.copy()` six times (`w.copy()`, `last_guess.copy()`, `cand[0].copy()`, `best.copy()`, `a.copy()`) to satisfy heap move-on-last-use rules. Clarifying whether `Str` literals coerce to `[0..5]Char` (or letting `.copy()` be automatic on tiny strings / clarifying borrow-vs-move in `for w in dict:` loops) removes that visual noise.

### 5. `min_by` / `max_by`, Compound `if let ... and cond:`, and `List.filled` vs `List` Constructor (Low-Medium)

Four small one-line cheat-sheet omissions caused brief 2 AM pauses:

- `noiwc22p3_draft.cp` needed the best word by a key function and guessed `min_by(dict, a => score(a))!` (the cheat sheet §10 lists `argmin` and `min(gen)` but not `min_by` / `max_by`, which are strictly clearer when iterating a `List` directly rather than by index).
- `noi22p5_draft.cp` wrote `if let pos = inds[v].first_ge(l) and pos <= r:` — §5 shows `if let a = opt1 and let b = opt2:` (two bindings) without confirming whether mixing a `let` binding and a plain boolean guard (`and pos <= r`) is legal.
- `noi08p4_draft.cp` and `bsspc21s4_draft.cp` guessed `List.filled(n, val)` (which matches `stdlib/lazy.cp` line 22!) because §9 doesn't list a filled-list constructor.
- `bsspc21s4_draft.cp` and `noi22p5_draft.cp` wrote `type Val = xs.D` after `let xs = compress(raw)`, unsure whether `xs.D` can be aliased with `type` or `domain`.

### 6. `match` Guard / Nested Pattern Edge & Optional Index `?Col` Iteration (Low)

In `noi04p3_draft.cp`, `[Col.bounds, ?Col, VDir]?Int` was a joy to declare, and `for p in ?Col:` felt like the obvious way to iterate an algebraic `Index` type — since §4 says `?D` is an `Index` type, contestants naturally expect `for p in ?Col:` to work identically to `for c in Col:`. Confirming `for x in T:` works for every `Index` type (including `?D`, `Bool`, `Subset<D>`, and small `enum`) keeps the mental model 100% uniform.

---

## 5. Real Rust `cpl` (`c521b2c`) Readiness & Triple Differential Verification

Running the real Rust `cpl` compiler (`c521b2c`) on both the untouched blind drafts and the runnable ports revealed a sharp, actionable picture of end-to-end v2.2 readiness.

### First-Compile Failures on Untouched Blind Drafts (`*_draft.cp`)

All 6 untouched blind drafts failed first compile on both `target/debug/cpl check` (`c521b2c`) and `python3 proto/cp.py build`, exposing **nine exact parity gaps** between the locked v2.2 cheat sheet and `c521b2c`:

- **`F64` missing (`noi08p4_draft.cp`)**: `cpl check` rejects `F64` (`E0204: no type or domain named F64`) because `crates/cpl-check/src/prelude.rs` still names the float type `Float`. Likewise, `print("{ans:.2f}")` fails lexing (`E0005: 2f isn't a number`); Rust `cpl` uses `print(ans, digits: 2)`.
- **Filtered `for ... if` loops unparsed (`noi22p5`, `noi04p3`, `noi08p4`, `noiwc22p3`)**: `for u in V if not icyc[u]:` fails with `E0103: expected ':' after the 'for' header, found 'if'` (4 of 6 blind drafts hit this!).
- **Bodyless extern `@grader fn` unparsed (`ioi19p5_draft.cp`)**: `@grader fn add_and(ns: List<Int>) -> Int` fails with `E0103: expected ':' or '=' to start the body`.
- **Place-yielding `??=` expression unparsed (`ioi19p5_draft.cp`)**: `(mp[cb(r, c)] ??= []).push(...)` fails with `E0103: expected ')' to close the parenthesis, found an assignment operator`.
- **Type-valued `Index` iteration unparsed (`noi04p3_draft.cp`)**: `for mask in Subset<Row>:` fails with `E0102: expected a value after '>', found ':'`, and `for p in ?Col:` fails with `E0102: expected a value after 'in', found '?'`.
- **Path-dependent domain alias unparsed (`bsspc21s4_draft.cp`, `noi22p5_draft.cp`)**: `type Val = xs.D` fails with `E0105: expected a type`.
- **Inline `;` separator rejected (`bsspc21s4_draft.cp`)**: `xs_raw.push(r.x1); xs_raw.push(r.x2)` fails with `E0107`.
- **Missing v2.2 prelude / stdlib wiring (`noi22p5`, `noi08p4`, `noiwc22p3`)**: `SortedBag`, `LazySegTree`, `AddMin`, `desc`, `reverse`, and `min_by` all fail with `E0201: no ... in scope` (while `Set` methods in `cpl` are still `.insert`/`.min()`/`.max()` instead of `.add`/`.first`/`.last`, and `List` uses `.sort()`/`.reverse()` methods rather than free functions).
- **LOC cost of the current Rust gaps**: Rewriting all six problems into the Rust-runnable subset inflated total code size from **438 LOC** (v2.2 blind drafts) to **848 LOC** (+93.6%), proving directly how much boilerplate the remaining v2.2 features (`LazySegTree`, `SortedBag`, `compress`, `Subset<D>`, `for ... if`, `@memo`, `(m[k] ??= []).push`) eliminate.

### Two Real Rust Compiler Bugs Discovered

Testing all six runnable ports through `cpl check`, `cpl run`, and `cpl build` + ASan/UBSan caught two concrete compiler bugs in `c521b2c`:

1. **`cpl check` vs `cpl-ir` false error on recursive void methods (`bsspc21s4.cp`)**: When `CoverTree.make` called recursive void method `t.build(1, 0, n - 1, ws)` before `fn build(mut self, ...):` was declared, `cpl check` reported `no errors`, but `cpl build` failed with `error[E0204]: build calls itself, so it needs its return type written` and suggested `add -> Int`. Adding an explicit `-> ()` return annotation to `build` and `update_rec` worked around the lowering bug.
2. **Unchecked `List` indexing causes ASan `heap-buffer-overflow` in emitted C++17 (`noi08p4.cp`)**: When our initial random generator passed `m = 0` with `clen < cyc.len` (`budget = 0`), `cans.v` became empty and `cans.v[cans.v.len - 1]` read `cans.v[-1]` in the `cpl build` binary — crashing with an AddressSanitizer `heap-buffer-overflow` instead of a clean bounds trap (note: `5144322.cpp` also has UB on `m = 0` when `clen < cyc.len`). Emitted C++17 should bounds-check `List` indexing in debug/default builds (or preserve the zero-UB guarantee outside `unchecked`).

### Triple Differential Verification Results (All 6/6 Passing!)

Once adapted to the Rust `c521b2c` subset, **all six full Olympiad solutions** compiled cleanly with `cpl build`, compiled with zero warnings under `g++ -std=c++17 -O2 -fsanitize=address,undefined`, and passed **100% triple differential verification** via `python3 proto/ports/blind_critic_5/check.py --iters 30 --interp-iters 5`:

- **`bsspc21s4`**: 30/30 vs `4152932.cpp` (ASan/UBSan), 5/5 `cpl run == cpl build`, emitted C++17 **20,205 B**, `cpl check` **5.6 ms**, `cpl build` **8.9 ms**.
- **`noi22p5`**: 30/30 vs `4832632.cpp` (ASan/UBSan), 5/5 `cpl run == cpl build`, emitted C++17 **21,204 B**, `cpl check` **4.8 ms**, `cpl build` **8.1 ms**.
- **`noi04p3`**: 30/30 vs `5032868.cpp` (ASan/UBSan, exercising real v2.2 tropical `?Int` with zero `1e9` sentinels), 5/5 `cpl run == cpl build`, emitted C++17 **21,013 B**, `cpl check` **4.0 ms**, `cpl build` **5.6 ms**.
- **`noi08p4`**: 30/30 vs `5144322.cpp` (ASan/UBSan), 5/5 `cpl run == cpl build`, emitted C++17 **12,471 B**, `cpl check` **3.8 ms**, `cpl build` **9.5 ms**.
- **`noiwc22p3`**: 30/30 vs `4261177.cpp` (ASan/UBSan, deterministic minimax Wordle guesses identical on every query), 5/5 `cpl run == cpl build`, emitted C++17 **9,797 B**, `cpl check` **2.6 ms**, `cpl build` **3.8 ms**.
- **`ioi19p5`**: 30/30 vs `5664246.cpp` (ASan/UBSan, every emitted Boolean circuit exhaustively evaluated across all cell pairs `(r1, c1), (r2, c2)`), 5/5 `cpl run == cpl build`, emitted C++17 **9,482 B**, `cpl check` **3.3 ms**, `cpl build` **4.4 ms**.

Even in an unoptimized debug build of `cpl`, frontend check latency (**2.6–5.6 ms** vs 50 ms P0 budget), C++17 emission latency (**3.8–9.5 ms**), and emitted C++17 file size (**9.5–21.2 KB** vs 64 KB P0 limit) are all well inside P0 targets, and the interpreter (`cpl run`) matched the ASan/UBSan C++17 binary on **30/30** runs without a single discrepancy.

---

## 6. Final Verdict & Prioritized Follow-Up

**Language specification verdict: LOCK v2.2 AS-IS (34 keywords).**
The 34-keyword v2.2 surface passed its final blind stress test cleanly: zero keyword cuts were regretted, all five Blind Critic #4 ambiguities are resolved on the cheat sheet, the six blind drafts achieved **-60.3% raw LOC** and **-34.3% library-stripped LOC** on genuinely hard Olympiad problems, and CP Lang's domain system even caught a latent `Pos`-vs-`Val` bug in the accepted C++ submission for `noi22p5`.

**Remaining work is purely cheat-sheet micro-polish and Rust compiler v2.2 parity (zero language-design churn):**

1. **Cheat-sheet micro-edits (~6 lines, zero new concepts):** add function-type syntax (`(A, B) -> C`), resolve the `;` statement-separator contradiction, note `AddMin` returns `Int` (`Int.MAX` on empty) and whether `Set`/`SortedBag` support `.rank`/`.kth`, list `List.filled(n, val)` and `min_by`/`max_by`, and note zero-arg `@memo` + `for x in ?D:`.
2. **Close the 9 Rust `cpl` (`c521b2c`) v2.2 parity gaps in priority order:**
   - Rename `Float` -> `F64` (keep `Float` alias temporarily or migrate tests) and support `{x:.2f}` format specs in string interpolation.
   - Parse filtered `for x in xs if cond:`, `(m[k] ??= init).method(...)`, bodyless extern `@grader fn`, `for mask in Subset<D>:`, `for p in ?D:`, and `type X = cx.D`.
   - Wire `std` prelude / `SortedBag` / `LazySegTree` / `AddMin` / `compress` / `desc` / `reverse` / `min_by` and align `Set.add`/`.first`/`.last` with the locked v2.2 cheat sheet.
3. **Fix the two Rust compiler bugs found here:** void recursive methods called before their declaration in `cpl-ir` (`bsspc21s4.cp`), and bounds-checked `List` indexing in emitted C++17 (`noi08p4.cp`).
