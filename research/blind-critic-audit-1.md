# Blind Contest-Pressure Critic Audit #1

## Method and Corpus

To measure first-draft friction and mental overhead under simulated contest pressure, I read only the user-facing documentation (`site/content/*.md`, `site/examples/*.cp`, and `plan/10-language-tour.md`, `plan/16-io.md`, `plan/17-syntax.md`, `plan/20-stdlib.md`) before writing all eight first drafts and all mid-contest pivots. I did not read `proto/cpc/` or `crates/` until every first draft and pivot error had been logged; compiler internals were inspected afterward only to confirm root causes and finalize working ports.

Eight unseen, medium-to-hard submissions from `corpus/submissions/` (none present in `proto/ports/` or `scout_batch_*`) were selected to span distinct competitive-programming domains:

- `4331904`: minimum-token string segmentation DP with lexicographic reconstruction (Pivot 1: longest-match greedy to suffix DP).
- `4342005`: multi-source bounded-distance grid BFS with color tie-breaking.
- `4553296`: directed-graph fixed-point propagation (SPFA-style queue with deduplicated predecessors).
- `4811954`: IOI 2009 Salesman day-grouped DP with two range-max segment trees (Pivot 2: quadratic DP to `O(N log N)` `SegTree` DP).
- `5044895`: cyclic permutation min-max shift distance (Pivot 3: quadratic shift scan to binary search + difference array).
- `3540475`: complete 1D coordinate MST with free edges (Pivot 4: small-to-large DSU + ordered `Map` predecessor/successor to builtin `Dsu<V>` + 2-tracker linear Boruvka sweeps).
- `3767355`: bandwidth-`k` Laplacian determinant via profile bitmask DP modulo `65521`.
- `3757433`: constructive tree pebbling with recursive DFS and tagged operation output.

Every final port under `proto/ports/blind_critic_1/` compiles cleanly through `proto/cp.py build` and `g++ -std=c++17 -O2 -fsanitize=address,undefined` and passed randomized differential testing against the original C++ binary (460/460 cases total: 60 each on seven problems and 40 on `3767355`, zero mismatches, zero ASan/UBSan findings).

### Dual LOC Summary (CP Lang vs. Raw C++ and Library-Stripped C++)

- `4331904` (string DP): 46 CP LOC vs. 71 raw C++ LOC (-35%) and 34 stripped C++ LOC (+35% longer due to opaque `Str`).
- `4342005` (grid BFS): 37 CP LOC vs. 86 raw (-57%) and 49 stripped (-24%).
- `4553296` (graph fixed-point): 36 CP LOC vs. 88 raw (-59%) and 57 stripped (-37%).
- `4811954` (Salesman DP): 83 CP LOC vs. 133 raw (-38%) and 102 stripped (-19%, or +34% vs. 62 LOC if C++ had a library `SegTree`).
- `5044895` (cyclic shift): 31 CP LOC vs. 92 raw (-66%) and 49 stripped (-37%).
- `3540475` (Boruvka MST): 71 CP LOC vs. 95 raw (-25%) and 55 stripped (+29% longer due to missing `Map.lower_bound`).
- `3767355` (bitmask DP): 50 CP LOC vs. 110 raw (-55%) and 69 stripped (-28%).
- `3757433` (tree pebbling): 79 CP LOC vs. 126 raw (-37%) and 85 stripped (-7%).

Totals across all 8 problems: 433 CP LOC vs. 801 raw C++ LOC (-46%) and 500 library-stripped C++ LOC (-13%). The two problems where CP Lang was longer than stripped C++ (`4331904` and `3540475`) correspond directly to the two largest missing stdlib/core capabilities below.

---

## Findings Ranked by Mental Overhead

### 1. Silent Heap-Use-After-Free in Tuple Destructuring (`4342005`) — Critical

In `4342005`, because `Deque` did not exist, I used the standard `List` + `head` cursor BFS pattern:

```cp
var head = 0
while head < q.len:
    let (r, c, color, d) = q[head]
    head += 1
    if grid[r, c] == 0:
        grid[r, c] = color
    if let nr = r.offset(-1): push(nr, c, color, d + 1)
    if let nr = r.offset(1):  push(nr, c, color, d + 1)
    if let nc = c.offset(-1): push(r, nc, color, d + 1)
    if let nc = c.offset(1):  push(r, nc, color, d + 1)
```

This compiled cleanly through `proto/cpc` and `g++ -O2` with zero warnings, then produced a silent wrong answer on the very first sample (`3 2 0 1` instead of `3 2 1 1`). Recompiling with `-fsanitize=address,undefined` exposed an immediate heap-use-after-free:

```text
ERROR: AddressSanitizer: heap-use-after-free
READ of size 4 ...
freed by thread T0 here:
    ... std::vector<Tup0>::_M_realloc_insert ...
```

Post-audit inspection confirmed the root cause in `proto/cpc/compiler.py:2281-2284`: `bind_pattern` emits `const auto& _d0 = q[head]; const auto& r = _d0.f0; ...` without recording a borrow hold on `q`. The first `push` inside the loop body reallocates `q`'s backing vector, turning `r`, `c`, `color`, and `d` into dangling references for the remaining three neighbor pushes.

Under contest pressure, this is the worst possible failure mode: idiomatic CP Lang code compiles cleanly, looks completely safe, and silently corrupts memory on vector growth. Either tuple destructuring from an indexed container must copy small values (or copy by default for `let`), or the borrow checker must reject mutating `q` while `r`/`c`/`color`/`d` are live.

### 2. `Str` Is Effectively Unusable for String Problems (`4331904`) — Critical

After reading `site/content/strings.md` and `plan/17-syntax.md`, I naturally wrote:

```cp
let s: Str = read()
domain D = 1..=n
let dict: [D](c: Char, x: Str) = read()
...
if i + x.len <= m and s[i..i + x.len] == x:
```

Compiler error:

```text
4331904_draft1b.cp: error: line 14: cannot index Str
```

Thirteen blind probes against `Str` all failed:

- `s.raw` -> `Str has no .raw`
- `for ch in s:` -> `cannot iterate Str`
- `s[0]` -> `cannot index Str`
- `s.substr(0, 1)` -> `Str has no method substr`
- `s.chars()` / `s.chars` -> `Str has no method chars` / `Str has no .chars`
- `s.at(0)` -> `Str has no method at`
- `s.slice(0, 1)` -> `Str has no method slice`
- `s.bytes()` -> `Str has no method bytes`
- `s.copy()` -> `Str has no method copy`
- `s.join(ans)` -> `Str has no method join`
- `s + s` -> `` `+` needs Int operands, got Str and Str ``

`Str` in `proto/cpc` only supports `.len`, `==`, and `print`. Reading a word whose characters you can actually inspect requires the `[domain S]Char = read()` syntax—which only works on standalone `let` bindings, not inside arrays of tuples like `[D](c: Char, x: Str)`. To port `4331904`, I had to read each dictionary word into a loop-local `[domain W]Char`, copy its bytes one by one into `var words: [D]List<Char> = []`, and write a manual `matches_at` character loop. That single limitation turned a 34-line C++ solution into a 46-line CP Lang solution.

### 3. Defining `struct SegTree` or `struct Dsu` Crashes the Compiler, and Structs Containing `?T` Fail `g++` (`4811954`, `3540475`) — Critical

Because `site/content/stdlib.md` never documents constructor signatures or method lists for `SegTree` or `Dsu`, I wrote my own structs during `4811954` (Pivot 2) and `3540475` (Draft 1):

```cp
struct SegTree:
    sz: Int
    v: List<?Int>
    fn update(mut self, pos: Int, x: Int):
        ...
```

Both `struct SegTree:` and `struct Dsu:` crashed `proto/cpc` with an uncaught Python exception:

```text
  File ".../proto/cpc/compiler.py", line 400, in resolve_type
    d = self.resolve_type(tn.args[0])
IndexError: list index out of range
```

`resolve_type` hardcodes `"Dsu"` and `"SegTree"` before consulting user-defined structs and indexes `tn.args[0]` without checking length.

Worse, after I worked around the crash by renaming `SegTree` to `MaxTree`, `proto/cp.py build` succeeded (`wrote 4811954_pivotb.cpp`), and then `g++` exploded with 40 lines of C++ template errors:

```text
/usr/include/c++/11/bits/predefined_ops.h:45:23: error: no match for 'operator<' (operand types are 'const Opt<long long int>' and 'const Opt<long long int>')
```

The compiler unconditionally synthesizes `operator<` on every user `struct` via `std::tie` over all fields, even when the struct is never ordered, and `Opt<T>` does not implement `operator<`. Putting `?Int` or `List<?Int>` inside any user `struct` breaks C++ compilation—directly punishing the exact sentinel-free `?Int` idiom the docs tell users to adopt.

### 4. `Map` and `Set` Lack Ordered Predecessor/Successor Queries and Documentation (`3540475`) — High

`site/content/stdlib.md` states that `Map` and `Set` are ordered collections, but never lists their methods. In `3540475`, my first draft needed the smallest element of `Set<(Int, V)>`, key deletion, and `upper_bound` / predecessor lookup in `Map<Int, V>`:

```cp
let (_, u) = comps.first()!
vals.erase(arr[a])
if let (k, tv) = vals.ceil(arr[a]):
```

First error:

```text
3540475_draft1c.cp: error: line 55: no method first on Set<(f0: Int, f1: V)>
```

Blind probing showed `Set` uses `.min()`, `.max()`, and `.remove(x)` (not `.first()` or `.erase()`), while `Map` only has `.get()`, `.remove()`, and `.clear()`. Every single ordered-query name I guessed (`lower_bound`, `upper_bound`, `floor`, `ceil`, `ge`, `gt`, `le`, `lt`, `prev`, `next`) failed on both `Set` and `Map`. Post-audit inspection confirmed `proto/cpc` has no ordered predecessor/successor operations on `Map` or `Set` at all—forcing a complete algorithmic pivot on `3540475`.

### 5. Doc-vs-Compiler Syntax Drift Causes Repeated First-Draft Rejections (`4331904`, `4342005`, `4553296`, `4811954`, `5044895`, `3767355`, `3757433`) — High

Every single one of the eight first drafts failed on its first compile, and six distinct failures came directly from syntax or names shown in the user-facing docs that `proto/cpc` rejects:

1. **Tuple `read()` (`4331904`)**: `let (n, m): (1..=100, 1..=1000) = read()` -> `error: line 1: unknown name read`. You must write separate `let n: ... = read()` statements.
2. **Binary search name (`5044895`)**: `course-verification.md` teaches `first_true(1..=1e9, x => ...)`, which fails with `error: line 30: unknown name first_true`. The implemented spelling is `(0..=n).first_where(x => ...)` (method on a range, not a free function).
3. **Inline `if` expression (`3767355`, `3757433`)**: `plan/17-syntax.md` documents `if cond then a else b`, which fails with `error: line 35:42: expected ':' (at 'then')`. The parser requires colons: `if cond: a else: b`.
4. **`enum` and `match` (`3757433`)**: `plan/17-syntax.md` shows positional enum variants (`L(V)`) and `case` arms (`case Op.L(u):`). That produced three sequential parse errors before compiling:
   - `L(V)` -> `error: line 13:8: expected ':' (at ')')` (variants must use named fields `L(u: V)`).
   - `case Op.L(u):` -> `error: line 84:14: expected ':' (at 'Op')` (no `case` keyword).
   - `Op.L(u):` -> `error: line 84:11: expected ':' (at '.')` (bare variant name `L(u):` only).
5. **Unimplemented collection types & constructor verbosity (`4342005`, `4553296`)**: `Deque` fails with `unknown type Deque`; `HashSet` fails with `unknown type HashSet`; `var preds: [V]Set<V> = Set()` fails with `unknown name Set`, forcing repetition of the type parameter (`Set<V>()`).
6. **`mut` placement (`4342005`, `4811954`)**: `mut fn push(...)` fails with `expected end of line (at 'fn')` (nested functions capture and mutate outer `var`s without a modifier); `for (day, mut cur) in by_day:` fails with `expected ')' (at 'cur')`.

None of these are deep semantic issues, yet together they cost 2–5 compile cycles on almost every problem because a contestant cannot trust the cheat sheet.

### 6. Borrow-Checker False Positive on Small-to-Large Merge + Misleading Help Text (`3540475`) — Medium

Inside `UnionFind.merge`, right after guarding `if u == v: return`:

```cp
for x in self.comps[u]:
    self.comps[v].push(x)
self.comps[u].clear()
```

Compiler error:

```text
3540475_draft1b.cp: error: line 30: `self.comps[v]` can't be changed here: the `for` loop on line 29 is going over `self`, and that lends it for reading until the loop ends
  help: loop over positions instead (`for i in 0..self.len`), or over a copy (`for x in self.copy()`)
```

Two issues compound here. First, the borrow checker tracks the hold at the root `self` rather than knowing `u != v` makes `self.comps[u]` and `self.comps[v]` disjoint, forcing an unnecessary `.copy()` allocation on every small-to-large merge. Second, the help hint interpolates the root name `self` (`0..self.len` / `self.copy()`), which does not even typecheck on a struct.

### 7. Missing Bitwise Staples and Generic-Tuple Parsing Bug (`3767355`, `4811954`) — Medium

In `3767355`, bitmask DP immediately hit four small-but-jarring gaps:

```text
3767355_draft1b.cp: error: line 29: unknown name popcount
3767355_draft1c.cp: error: line 29: Int[0..=1023] has no method `popcount`
3767355_draft1d.cp: error: line 29: Int[0..=1023] has no method `count_ones`
3767355_draft1e.cp: error: line 5:13: expected an expression (at '=')   // on `x >>= 1`
```

Every bitmask DP problem uses popcount (plus `ctz`/`clz` and `>>=`/`|=`/`&=`). Having to hand-write a `while` loop for `popcnt` with `x = x >> 1` breaks flow immediately.

Separately, in `4811954`, writing a nested tuple inside a generic constructor expression failed to parse:

```cp
var by_day: Map<Int, List<(l: Int, c: Int)>> = Map<Int, List<(l: Int, c: Int)>>()
// error: line 12:55: expected end of line (at ',')
```

Extracting `type FairItem = (l: Int, c: Int)` worked around it.

### 8. Optional Ergonomics and Map Auto-Initialization Friction (`4811954`) — Medium

Two small patterns repeated heavily in `4811954`:

- **Map lookup-or-insert**: `by_day[t].push(...)` compiled cleanly in Draft 1b, then crashed at runtime with `line 11: key not in map`. Unlike C++ `map[k].push_back(...)` (or a `.entry(k).or_default()`), every bucket insertion required a 3-line guard (`if by_day.get(x.t) == none: by_day[x.t] = []`).
- **Combinator boilerplate for `SegTree<Pos, ?Int>`**: Although scalar `cur max= cand` works cleanly on `?Int`, passing a monoid combiner to `SegTree<Pos, ?Int>(none, ...)` required writing a 6-line `opt_max(a: ?Int, b: ?Int) -> ?Int` helper because `max(a, b)` does not lift over `?Int`.

---

## Mid-Contest Pivot Notes

### Pivot 1 (`4331904`): Longest-Match Greedy -> Suffix DP with Reconstruction

Draft 1 attempted a greedy longest-match scan. Recognizing that a shorter prefix can unlock a fewer-token suffix (e.g. `dict = {(a, "0"), (b, "01"), (c, "10")}`, `s = "010"`), I pivoted to backward DP over `S.bounds`.

- **What was fast**: Adding `var dp: [S.bounds]?Int = none`, `dp[S.bounds.last] = 0`, `for i in S.rev:`, and `dp[i.l] min= 1 + nxt` took under a minute and read cleaner than C++.
- **What was slow**: Discovering `Str` could not be indexed forced rewriting the entire input section and every string comparison into `[domain W]Char` + `List<Char>` + manual `matches_at`.

### Pivot 2 (`4811954`): Quadratic Day-Pair DP -> `O(N log N)` Two-`SegTree` DP

Draft 1 maintained a flat `dp` over visited fairs and scanned all previous fairs per new fair (`O(N^2)`). Pivoting to the IOI 2009 Salesman `O(N log N)` two-segment-tree solution (splitting `loc <= cur` vs. `loc >= cur` plus two linear same-day sweeps) was the roughest refactor of the audit:

- Writing a custom `struct SegTree` crashed the compiler (`IndexError`); renaming to `MaxTree` failed `g++` because `List<?Int>` lacked `operator<`.
- Only after the blind phase did inspecting `proto/cpc` reveal that `SegTree<Pos, ?Int>` was already built in (`SegTree<D, T>(id, (a, b) => ...)`). Once switched to builtin `SegTree<Pos, ?Int>`, the DP itself was clean, though repeated `Pos.at(loc)` conversions and `opt_max` added friction.

### Pivot 3 (`5044895`): Naive `O(N^2)` Rotation Scan -> Binary Search + Difference Array

Draft 1 brute-forced all `n` cyclic shifts in `O(N^2)`. Compiling Draft 1 produced a standout compiler win:

```text
5044895_draft1.cp: warning: line 10: this loop nest can run ~4.0e+10 times at the largest declared input
```

Pivoting to binary search on the answer `x` + wrap-around difference array (`var diff: [V.bounds]Int = 0`) was fast overall, with two minor snags: `first_true(0..=n, ...)` from the course failed to resolve (actually `(0..=n).first_where(...)`), and clamping wrapped `Int` interval endpoints back into `V.nth(l).l` / `V.nth(r).r` added ceremony compared to plain `diff[l] += 1; diff[r + 1] -= 1`.

### Pivot 4 (`3540475`): Ordered `Map` Predecessor/Successor -> Builtin `Dsu<V>` + 2-Tracker Linear Boruvka

Draft 1 tried to mirror the C++ small-to-large DSU + `std::map::upper_bound` approach. Hitting the `struct Dsu` crash, the `self.comps[u]` borrow rejection, and the total absence of ordered predecessor/successor queries on `Map` forced pivoting to pure Boruvka rounds over a single `order = verts.sorted_by(v => arr[v])` array using left-to-right and right-to-left 2-component trackers. Builtin `Dsu<V>()` (`dsu.find`, `dsu.unite`, `dsu.count`) and `.sorted_by(...)` made the pivoted algorithm fast and bug-free, though the duplicated forward/backward tracker loops were still 16 lines longer than the stripped C++ `std::map` version.

---

## What Worked Well

- **Static complexity warnings (`5044895`)**: Catching the `~4.0e+10` `O(N^2)` loop nest at compile time from the declared `1..=2e5` constraint is genuinely useful under contest pressure.
- **`Mod<65521>` (`3767355`)**: Worked on the first try including negative coefficients (`Mod<65521>(-1)`), completely eliminating C++'s `infmod` negative-modulo workaround.
- **Boundary domains (`V.bounds`, `.l`, `.r`, `S.rev`)**: Prefix sums (`4811954`), difference arrays (`5044895`), and reverse suffix DP (`4331904`) read cleanly with zero off-by-one confusion.
- **Scalar `?Int` with `min=` / `max=` (`4331904`, `4553296`, `4811954`)**: Replacing `1e9` / `-1e18` sentinels with `?Int` and `dp[i.l] min= cand` felt natural and eliminated sentinel-overflow bugs.
- **Exhaustive `enum` + `match` (`3757433`)**: Once the colon-based syntax was discovered, `enum Op` with `L(u: V)`, `B(u: V)`, `M(u: V, v: V)` compiled cleanly and matched the original output bit-for-bit on the first run.

---

## Prioritized Recommendations

1. **Fix `bind_pattern` UAF immediately (P0)**: Never emit dangling `const auto&` bindings into mutable containers (`let (r, c, ...) = q[head]`). Copy value/tuple bindings or enforce a borrow hold on `q` until the bindings go out of scope.
2. **Make `Str` a real indexed byte sequence (P0)**: Unify `Str` with `[domain S]Char` (indexing, slicing, iteration, `.copy()`, equality against slices, and reading `[D]Str` cleanly) so string problems do not require `List<Char>` workarounds.
3. **Fix `Opt<T>` `operator<` and builtin-name shadowing (P0)**: Implement `operator<` on `Opt<T>` (or only synthesize struct `operator<` when all fields are orderable), and never crash in `resolve_type` when a user defines `struct Dsu:` or `struct SegTree:`.
4. **Add ordered `Map`/`Set` predecessor/successor methods and `Deque` (P1)**: Ship `lower_bound`/`upper_bound` (or `first_ge`/`last_le`, matching `Array`) on `Map` and `Set`, plus `Deque<T>` and `popcount`/`ctz`/`clz` + bitwise compound assignment (`>>=`, `<<=`, `&=`, `|=`).
5. **Eliminate doc-vs-compiler drift with a single runnable one-page cheat sheet (P1)**: Sync `first_where` vs. `first_true`, `if ...: ... else: ...`, `enum`/`match` syntax, `Map`/`Set`/`SegTree`/`Dsu` constructors and methods, and `let` vs. `read()` rules, and verify every cheat-sheet snippet in CI against the compiler.
6. **Improve ergonomics for common CP idioms (P2)**: Support `Map` bucket initialization (`m.entry(k)` or default-on-index for `List` values), lift `min`/`max` or provide `.or_else`/monoid helpers for `SegTree<D, ?Int>`, and fix borrow-error help hints so they name the actual field (`self.comps[u]`) rather than `self`.
