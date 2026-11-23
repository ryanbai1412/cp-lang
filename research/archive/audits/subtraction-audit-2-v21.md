# Stage 3: Subtraction & Concept-Budget Audit #2 (CP Lang v2.1)

*Adversarial simplification audit of CP Lang v2.1 (`research/archive/architecture/language-architecture-v2.1.md`, `docs/one-page-cheat-sheet.md`, `decisions.md`, `research/archive/audits/subtraction-audit-1.md`, `crates/cpl-syntax/src/token.rs`, 21 `stdlib/*.cp` files, 28 `examples/local-lib/**/*.cp` files, and all 923 `.cp` files in `proto/ports/` — 642 canonical ports plus 280 `today/` interpreter-workaround mirrors and 1 helper).*

CP Lang v2.1 held the line on its headline counters — zero new reserved keywords, zero new traits, zero new numeric types — while resolving real multi-feature collisions uncovered across Batches 21–41. Yet holding the keyword count flat masked a quieter form of bloat: **surface-area creep inside methods, sub-rules, and special cases**. One-off helpers (`@grader const`, `Marks<D>`, `D.chunks(k)`, `D[..k]`, `r1 & r2`, `.mirror`, `.flipped_rows`/`.flipped_cols`, `.absorb()`, `rotate_right`, `prod`, `bsearch_first_some`, `slide_until`), a second iteration borrow mode (`for mut`), over-broad tropical optional rules (`sum` absorbing `none` while `min` skips `none`), and five keywords that earned zero or negative payoff (`defer`, `where`, `inline`, `lend`, hard keyword `law`) all crept into the specification and pushed `docs/one-page-cheat-sheet.md` to 242 lines (20.3 KB).

This audit applies two tests to every construct in v2.1:

1. **Supply-side frequency:** how often does the construct actually appear across the 642 canonical `.cp` ports, 21 `stdlib/` files, and 28 `examples/local-lib/` files?
2. **Demand-side frequency:** because many ports predate Wave 3 (`048db18`), zero direct hits on a brand-new v2.1 feature do not automatically condemn it. For every new feature, we also scan all 642 canonical ports for the **exact manual workaround** the feature claims to replace (such as `if let` + arithmetic + `min=`/`max=` for Tropical Optionals, `.len` swap + `.clear()` for `.absorb()`, generation-stamped `seen[v] == tag` for `Marks<D>`, or `/ B` block loops for `D.chunks(k)`).

Applying the cuts below shrinks CP Lang from **40 reserved keywords to 35** (31 contestant + 4 library-author), from **8 + 4 traits (claimed "8 + 3") to 7 core + 2 library-author**, trims 16 niche methods and sub-rules from the core mental surface, and pulls `docs/one-page-cheat-sheet.md` back down to roughly ~205 lines.

---

## 1. Corpus Frequency Snapshot & Spec Drift

Across all **642 canonical ports** in `proto/ports/` (excluding the 280 `today/` mirrors and `hashing.cp`), usage breaks sharply into three tiers:

**Tier 1 — High-frequency core (the real language contestants write every day):**
- Core bindings & control flow: `let` (641 files / 4,613 occ), `in` (623 / 3,026), `for` (620 / 2,947), `var` (577 / 2,661), `if` (577 / 2,556), `fn` (440 / 1,049), `else` (372 / 767), `and` (252 / 496), `while` (232 / 370), `none` (215 / 634), `return` (195 / 426), `true`/`false` (169/161), `not` (148 / 268), `or` (126 / 190), `elif` (94 / 136), `continue` (81 / 121), `break` (63 / 84), `match` (42 / 55).
- Core type & domain system: `Int` (535 / 3,124), `domain` (481 / 839 — **75% of all ports**), `type` (173 / 218), `use` (144 / 230), `const` (112 / 130), `mut` (74 / 203), `struct` (63 / 81), `enum` (36 / 36), `self` (31 / 309), `assert` (17 / 71), `assume` (6 / 8).
- Core operators & stdlib workhorses: `min=`/`max=` (122 / 245), `sum(` (119 / 188), `??` (103 / 140), `by:` named arg (78 / 97), `Mod<P>` (64 / 67), `Rooted` (53 / 101), `@memo` (45 / 49), `Graph.` (34 / 35), `Set<` (34 / 55), `Dsu` (30 / 53), `Map<` (29 / 35), `SegTree` (26 / 39), `@brute` (26 / 26), `@multitest` (24 / 24), `Subset<` (21 / 27), `LazySegTree` (20 / 36), `MinHeap` (19 / 19), `prefix(` (19 / 24), `@grader` (18 / 28), `first_ge` (17 / 22), `HashMap` (16 / 18), `Fenwick` (14 / 31), `Deque` (12 / 13), `dijkstra` (12 / 14), `Pt` (11 / 66), `compress` (11 / 12), `@tags` (11 / 11), `bfs` (9 / 10), `Comb<` (9 / 9), `U64` (8 / 58), `@checker` (8 / 8), `desc(` (8 / 11), `Dir4` (7 / 18), `.take(` (7 / 8), `scan(` (7 / 8), `reverse(` (7 / 7), `@interactor` (7 / 7), `F64` (6 / 21), `MaxHeap` (6 / 9), `SortedBag` (6 / 6), `bsearch_first` (6 / 6), `runs(` (6 / 6), `Bitset` (5 / 12), `@test` (5 / 10).

**Tier 2 — Low-frequency but justified specialized tools (2–5 files, or essential library hooks):**
- `Sieve` (4), `suffix_array` (4), `.transposed` (4), `last_le` (4), `nested @memo` (4), `@memo(hash)` (3), `SparseTable` (3), `FlowGraph` (3), `HashSet` (3), `dedup` (3), `two_color` (3), `condense` (3), `ceil_div` (3), `ext_gcd` (3), `topo(` (3 total: 2 bare + 1 `topo::<`), `I128` (2), `bsearch_last` (2), `lcp_array` (2), `prefix_function` (2), `virtual_tree` (2), `biconnected_components` (2), `articulation_points` (2), `scc` (2), `max_weight_matching` (2), `divisors` (2), `yield` (11 files in ports, plus core generator mechanism), `@get` (4 in ports + 44 across 12 `stdlib/*.cp` files).

**Tier 3 — Near-zero or zero usage (prime deletion/merge targets):**
- **Keywords at 0–1 uses:** `defer` (1 file in ports, 0 stdlib, 0 local-lib), `inline` (0 ports, 0 stdlib, 0 local-lib), `law` (0 ports, 4 in `stdlib/algebra.cp`, 0 local-lib), `lends` / `lend` (0 ports, 0 stdlib, 1 in `examples/local-lib/math/matrix.cp`), `where` for trait bounds (0 ports, 0 stdlib, 0 local-lib), `unchecked` (0 ports — explicitly retained as P1 escape hatch).
- **v2/v2.1 syntax & helpers at 0–1 uses:** `for mut` (0 direct / 5 manual index-mutation loops), `D.chunks(k)` (0 direct / 4 manual `/ B` files), `D[..k]` (0 direct / 1 motivating file), `r1 & r2` range intersection (0 direct / 1 motivating line), `.mirror` (1 file), `.flipped_rows` (0), `.flipped_cols` (0), `Marks<D>` (0 direct / 0 generation-stamped visited arrays / 2 `.fill(false)`), `.absorb()` (0 direct / 1 `.len`-swap+`.clear()`), `topo::<V>` (1), `topo_prefix` (0), `@grader const` (1 novelty problem), `@memo(flat)` (0), `bsearch_first_some` (1), `slide_until` (1), `rotate_right` (0; `rotate_left` is 1), `prod(` (0), `sliding_min`/`sliding_max` (0), `MonoDeque` (1), `Steps` (1), `MinCostFlow` (1), `Dir8` (1), `z_function` (1), `two_edge_components` (1), `crt` (1), `mul_mod` (1), `TwoSat` (0), `Matrix` (0), `palindromes` (0), `shortest` (0), `mst` (0), `walk_back` (0), `next_permutation` (0), `floor_div` (0), `factorize` (0), `convex_hull` (0).

**Spec / lexer / corpus drift caught by the scan (action items for the next Language Architect pass):**
- `crates/cpl-syntax/src/token.rs` still reserves deleted v2 keywords `With` and `By` plus unused `Spaceship` (`<=>`), while missing v2 keywords `assume`, `lends`, etc.
- `decisions.md` ends with a stray git merge-conflict trailer line (`>>>>>>> 048db18 ...`) that should be cleaned up.
- Legacy pre-v2 spellings remain scattered across older ports: 73 files still use `where` (58 on `read()`/`let` refinements, 15 on type/spec constraints, 0 on trait bounds), 12 still call `first_where`, 11 still use `with`, 3 still use `ModR`, and 2 still use `.flipped`.
- `stdlib/dsu.cp` still defines two identical aliases (`components` on line 61 and `count` on line 65). Note also that commit `9031168` just implemented several of the v2.1 stdlib items audited below (`stdlib/marks.cp`, `stdlib/grid.cp` `flipped_rows`/`flipped_cols`, `stdlib/sparse.cp` `fold -> ?T`, and `stdlib/seq.cp` `prod`/`rotate_right`/`D.chunks`/`intersect`/`mirror`/`absorb`), making this audit directly actionable as a targeted subtraction pass over both the spec and `stdlib/`.
- `from` is used as a plain identifier in 55 canonical ports (118 occurrences, e.g. `fn route(u: V, from: ?V, ...)` in `aac5p3.cp` and `from: List<V>` in `stdlib/graph.cp`) and never as an import keyword — keep `from` permanently unreserved.

---

## 2. Detailed Scrutiny: Keep / Merge / Delete Decisions (with Corpus Evidence)

### 2.1 `defer` — **DELETE** (saves 1 contestant keyword: 40 -> 39)

Audit #1 deleted `with` and `undo` and kept `defer` as the sole scoped-cleanup mechanism. Twice the corpus size later (642 canonical ports), `defer` has **1 occurrence in 1 file** (`proto/ports/scout_batch_50/noi22p4.cp:47`), **0 in `stdlib/`**, and **0 in `examples/local-lib/`** — and inspecting actual backtracking ports reveals why contestants avoid it:

1. Its **only** use in the entire 642-file corpus (`proto/ports/scout_batch_50/noi22p4.cp:42–49`) mutates a 5-bit `Copy` `Subset<0..5>` bitmask solely to undo it on return:

Before (`proto/ports/scout_batch_50/noi22p4.cp:42–49`, 8 lines / 294 chars):
```cp
var taken = Subset<0..5>.empty
fn place(at: Int) -> Bool:
    if at == right.len: return true
    for c in 0..left.len if c not in taken and prune(left[c], right[at]):
        taken = taken.with(c)
        defer taken = taken.without(c)
        if place(at + 1): return true
    false
```

After (pure value parameter, 4 lines / 181 chars — **-4 lines, -113 chars, zero mutation, zero `defer`**):
```cp
fn place(at: Int, taken: Subset<0..5>) -> Bool:
    if at == right.len: return true
    any(place(at + 1, taken.with(c)) for c in 0..left.len
        if c not in taken and prune(left[c], right[at]))
```

2. In classic search-and-keep backtracking (`proto/ports/scout_batch_37/noip04p4.cp:24–29`), `defer` is an **active bug**: returning `true` MUST preserve `digit[l]` so the caller can print the solution at line 44! Writing `defer digit[l] = none` erases the winning assignment on `return true`:

```cp
// proto/ports/scout_batch_37/noip04p4.cp:24–29 — defer would destroy the answer!
for d in (0..n).rev if not used[d]:
    digit[l] = d
    used[d] = true
    if plausible() and solve(col, carry): return true
    digit[l] = none
    used[d] = false
```

3. In sequential-branch backtracking (`proto/ports/scout_batch_38/btoi17p4.cp:37–46`), `defer` cannot scope between two sibling branches (`In` then `Out`) because CP Lang has no anonymous blocks (`FRICTION 38.9`). And in rollback Mo's (`proto/ports/scout_batch_41/nccc9s4.cp:54–60`) or rollback DSU (`btoi20p3.cp`), rollback runs at the end of a non-returning loop body or between sibling recursive calls — one explicit `dsu.undo_to(snap)` or `for j in l..mid: cnt[id[j]] -= 1` line is clearer and never fires at the wrong exit.

**Verdict: DELETE `defer`.** Reclaims a top-level contestant keyword, removes scope-exit RAII codegen from the compiler, and eliminates a subtle backtracking trap.

---

### 2.2 Library-Author Keyword Cleanup: `where`, `inline`, `lend`, and `law` (`law disjoint`) — **DELETE 3, DEMOTE 1** (saves 4 library-author keywords: 39 -> 35)

Four of the eight library-author keywords never pull their weight as globally reserved words:

- **`where` — DELETE (39 -> 38).** Audit #1 already deleted `where` refinements on `read()` and `type` (collapsing them into `assume`), retaining `where` solely for trailing generic trait bounds (`fn f<T>() where T: Ord`). Across 642 canonical ports, 21 `stdlib/*.cp` files, and 28 `examples/local-lib/` files, trailing `where T: Trait` appears **0 times** — inline bounds `fn f<T: Ord>(...)` already handle 100% of generic constraints. Deleting `where` also eliminates the lingering ambiguity where 73 older ports still wrote `read() where ...`.
- **`inline` — DELETE (38 -> 37).** Appears **0 times** across all 642 ports, `stdlib/`, and `examples/local-lib/`. Every `lends` function already carries `lends` in its signature (which tells the compiler to desugar inline), `yield` generators already desugar to push loops, and zero-cost closure inlining is already CP Lang's default optimization contract. Reserving a separate hard keyword `inline` buys nothing.
- **`lend` — DELETE / MERGE into `return` & tail expression inside `lends` bodies (37 -> 36).** Inside a `lends` / `lends mut` function (`plan/19-index-and-lends.md` §3.4), every exit MUST lend exactly one place — ordinary `return value` is illegal there. Having two distinct hard keywords (`lends` in the signature AND `lend` in the body) is redundant. Using the tail expression (or `return mut ...` for early branches) inside a `lends` body is 100% unambiguous and shorter.

Before (`examples/local-lib/math/matrix.cp:15–16`, the **only** `lends` function in the entire repository):
```cp
pub fn at(self, i: Int, j: Int) lends mut Mod<P>:
    lend mut a[i][j]
```

After (zero `lend` keyword, matches CP Lang's tail-expression return rule):
```cp
pub fn at(self, i: Int, j: Int) lends mut Mod<P>:
    mut a[i][j]
```

- **`law` (including `law disjoint`) — DEMOTE from hard keyword to `@law` decorator + auto-derive forwarding disjointness (36 -> 35).** Across 642 canonical ports, `law` appears **0 times**; in `stdlib/`, it appears **4 times** in one file (`stdlib/algebra.cp:14, 15, 37, 38`, testing monoid/action axioms); `law disjoint` appears **0 times** in `stdlib/` and `examples/local-lib/`. The **capability** of disjointness proofs (enabling safe `swap(mut g.row(r1), mut g.row(r2))` and SMT/stress verification of algebraic laws) is worth keeping — it is core to CP Lang's zero-lifetime borrow checker. What does **not** earn a hard reserved keyword is the spelling `law`:
  1. For trivial single-index forwarding (`mut a[i][j]` in `matrix.cp`), the compiler should auto-derive disjointness from distinct index expressions (`(i1, j1) != (i2, j2)`), eliminating manual annotations on 95% of `lends mut` methods.
  2. For non-trivial views (`row(r)` / `col(c)` / `Monoid` axioms), spelling it `@law disjoint(i, j) = i != j` and `@law assoc(a, b, c) = ...` keeps 100% of the borrow-checker and SMT semantics, matches library-author `@get`/`@test`, and frees the identifier `law` from the reserved-keyword table.

---

### 2.3 Tropical Optionals (`?T` arithmetic, `min=`/`max=`, `==`/`!=`, `??` guard) — **KEEP CORE, DELETE `sum` absorber & `+=`/`-=` lifting**

This is the single most consequential design question in v2.1, so we scrutinize both sides rigorously.

**Why core Tropical Optionals (`min`/`max`/`min=`/`max=` identity + `+`/`-`/unary `-` absorber on `?Int`/`?F64` + `==`/`!=` auto-lift + unparenthesized-`??` compile error) CLEARLY EARN THEIR KEEP:**
- **122 of 642 canonical ports (19.0%)** use `?Int` / `?F64` / `?I128` / `?U64`.
- **42 canonical ports (75 call sites)** write the exact boilerplate `if let ...` + arithmetic + `min=`/`max=` guard that Tropical Optionals eliminate (`rpeng3.cp`, `yac6p6.cp`, `apio09p1.cp`, `dpq.cp`, etc.).
- **4 canonical ports** (`coci21c1p3.cp`, `ceoi22p5.cp`, `ccc22s5.cp`, `4811954.cp`) literally hand-define `fn plus(x: ?Int, y: ?Int) -> ?Int` or `fn shift(x: ?Int, d: Int) -> ?Int` to simulate Tropical Optionals manually.
- Without `+`/`-` absorption, `?Int` is strictly **more verbose** than C++'s `INF = 1e18` sentinel, driving contestants back to overflow-prone sentinels under contest pressure.

Before (`proto/ports/scout_batch_39/yac6p6.cp:74–86`, 12 lines — every transition and merge wraps an `if let` guard around `+`):
```cp
for k in 0..=MAXK:
    if let nr = r + 1 if nr <= cx:
        if let nxt = dp[nr, c, k]: dp[r, c, k] min= nxt + a[r, c]
        if k > 0: dp[r, c, k] min= dp[nr, c, k - 1]
    if let nc = c + 1 if nc <= cy:
        if let nxt = dp[r, nc, k]: dp[r, c, k] min= nxt + a[r, c]
        if k > 0: dp[r, c, k] min= dp[r, nc, k - 1]
...
for k in 0..=qry.kv:
    if let p1 = dp[qry.x1, qry.y1, k] and let p2 = dp2[dr, dc, qry.kv - k]:
        qans[qry.id] min= p1 + p2
```

After (Tropical Optionals — eliminates 3 `if let` guards, **-95 chars**, reads identically to the recurrence on paper):
```cp
for k in 0..=MAXK:
    if let nr = r + 1 if nr <= cx:
        dp[r, c, k] min= dp[nr, c, k] + a[r, c]
        if k > 0: dp[r, c, k] min= dp[nr, c, k - 1]
    if let nc = c + 1 if nc <= cy:
        dp[r, c, k] min= dp[r, nc, k] + a[r, c]
        if k > 0: dp[r, c, k] min= dp[r, nc, k - 1]
...
for k in 0..=qry.kv:
    qans[qry.id] min= dp[qry.x1, qry.y1, k] + dp2[dr, dc, qry.kv - k]
```

And in `proto/ports/scout_batch_35/coci21c1p3.cp:17, 27–31`, Tropical Optionals delete the custom `fn plus(x: ?Int, y: ?Int) -> ?Int: x? + y?` helper outright and turn `one = min(plus(one, off), plus(zero, lit)); zero = plus(zero, off)` into `one = min(one + off, zero + lit); zero = zero + off`. Meanwhile, `==`/`!=` auto-lifting (`if dq.last != m - i - 1:` in `aac4p6.cp:26`, `if digit[l] == total % n:` in `noip04p4.cp:33`) is intuitive and zero-ambiguity, and the compile error on unparenthesized `a[i] + max(...) ?? 0` prevents the exact Blind Critic #2 precedence bug without adding any syntax to memorize.

**Where v2.1 went too far — DELETE two high-overhead sub-rules:**
1. **DELETE `?Int`/`?F64` absorption on `sum(gen)`.** Across all 642 canonical ports, `sum(...)` over `?Int` appears **0 times** (the only match was `sum(1 for x in v if x == none)` summing plain `Int`s). Worse, making `min(gen)`/`max(gen)` *skip* `none` while `sum(gen)` *poisons* the entire result to `none` creates a nasty 2 AM contradiction between sibling builtins — and `sum([])` returning `0` while `min([])` returns `none` compounds it. When a problem genuinely wants an all-present sum, `if any(x == none for x in xs): none else: sum(x! for x in xs)` (or `fold(Monoid<?Int>(0, +), xs)`) is explicit and unmistakable.
2. **DELETE `+=` and `-=` lifting on `?Int`/`?F64`.** Silently turning `dp[u] += w` into a no-op when `dp[u]` happens to be `none` hides uninitialized-DP bugs with zero compile-time or runtime feedback, and appears cleanly in **0 canonical ports**. Keep lifting strictly on pure expressions (`+`, `-`, unary `-`) and `min=`/`max=`.

---

### 2.4 `for mut x in xs:` — **DELETE**

- **Data:** **0 direct uses** in 642 canonical ports; manual index-loop element mutation (`for i in 0..xs.len:` mutating `xs[i]`) appears in only **5 of 642 ports (0.8%)** — and 4 of those 5 are one-line element replacements or `.fill(false)` (`nccc10s4.cp:87`, `noip00p4.cp:33`). Only **1 file in 642** (`proto/ports/scout_batch_41/oly20practice139.cp:58`) calls a mutating method on elements in a loop.
- **Hidden cost:** Adding `for mut x in xs:` forces a second iteration borrow mode (mutable lending iterators, structural-mutation lock rules banning `push`/`pop` during the loop) across the compiler, traits, and cheat sheet.

Before (`proto/ports/scout_batch_41/oly20practice139.cp:58`, 54 chars — works today with zero extra language rules):
```cp
for i in 0..blocks.len: best max= blocks[i].query()
```

After (`for mut`, 40 chars — saves 14 chars in 1 file out of 642):
```cp
for mut b in blocks: best max= b.query()
```

**Verdict: DELETE `for mut x in xs:`.** Fourteen characters in 0.2% of problems is nowhere near worth a second iterator borrow mode in the compiler and cheat sheet.

---

### 2.5 `@grader const` and Uninitialized Shared Grader Domains — **DELETE** (keep `@grader` on `fn` and `struct`)

- **Data:** `@grader` overall is justified by **18 canonical ports (2.8%)** — 15 `@grader fn` (IOI/CCO/EGOI) and 3 `@grader struct`.
- **However**, `@grader const` appears in **1 file out of 642** (`proto/ports/scout_batch_40/tle18p5.cp:7`), whose C++ original (`corpus/submissions/5876770.cpp`) is a novelty DMOJ header-inclusion problem that solved LIS at compile time via recursive C++ template metaprogramming (`Node<D - 1>`). No standard IOI, CCO, NOI, or ICPC grader exports a bare compile-time global constant outside a function signature.
- Likewise, v2.1's special rule allowing a file-scope `domain Colour = 0..n` to stay uninitialized until the first `@grader fn` call is motivated by **1 file** (`proto/ports/scout_batch_37/ioi12p5.cp`, IOI 2012 Last Supper). It introduces a weird half-global, half-dynamic domain lifetime solely to avoid 2 local `domain` lines:

Before (`proto/ports/scout_batch_37/ioi12p5.cp:5–14`, requires magical late-bound file-scope domain `Colour`):
```cp
@grader fn GetRequest() -> Colour
@grader fn PutBack(c: Colour)
@grader fn ComputeAdvice(req: [0..n]Colour, n: Int, k: Int, m: Int): ...
```

After (zero special grader-domain lifetime rules, +2 local lines):
```cp
@grader fn GetRequest() -> Int
@grader fn PutBack(c: Int)
@grader fn ComputeAdvice(req: [0..n]Int, n: Int, k: Int, m: Int):
    domain Colour = 0..n
    ...
```

**Verdict: KEEP `@grader` on `fn` and `struct`; DELETE `@grader const` and uninitialized file-scope grader domains.**

---

### 2.6 `Marks<D>` (Generation-Stamped Visited Set) — **DELETE from `std`**

- **Data:** `Marks<D>` appears in **0 canonical ports** (it was just added to `stdlib/marks.cp` in `9031168`). On the demand side, across all 642 canonical ports, **0 files** write a generation-stamped visited array (`seen[v] == tag`), and only **2 files** call `.fill(false)` on a visited array (our broader `timer`/`stamp` scan hit 32 files, nearly all Euler-tour `tin[u]`/`tout[u]` timestamps).
- When a rare Kuhn's matching or multi-query BFS problem actually needs O(1) reset, the generation-stamp idiom is **3 lines of ordinary CP Lang** with zero hidden memory semantics:

Before (`Marks<V>` — new stdlib type on the cheat sheet):
```cp
var seen = Marks<V>()
for q in Q:
    seen.clear()
    if u not in seen: seen.add(u)
```

After (zero new stdlib type, explicit `[V]Int` memory footprint):
```cp
var seen: [V]Int = 0
for q in 1..=qs.len:
    if seen[u] != q: seen[u] = q
```

Notice that when iterating queries `1..=qs.len` (or vertices `1..=n`), you don't even need a separate `tag` variable — the outer loop counter *is* the stamp!

**Verdict: DELETE `Marks<D>` from `std` and `docs/one-page-cheat-sheet.md`** (optional 10-line helper in `examples/local-lib/` if desired).

---

### 2.7 `D.chunks(k)`, `D[..k]`, `r1 & r2`, and `.mirror` — **DELETE ALL FOUR**

Each of these four v2.1 additions solved a 1-file friction note by adding domain/range surface area with poor generality:

1. **`D.chunks(k)` — DELETE.** **0 direct uses** in 642 ports; manual sqrt-decomposition block division (`/ B` or `/ BLOCK`) appears in only **4 files (0.6%)** (`nccc9s4.cp`, `oly20practice139.cp`, `dmpg15g6.cp`, `noi17p2.cp`). Worse, in Rollback Mo's (`proto/ports/scout_batch_41/nccc9s4.cp:26–48`), queries are bucketed by `(l + B - 1) / B` and swept rightward past chunk ends — `D.chunks(B)` does not even fit Mo's! And in fixed-block sqrt decomposition (`oly20practice139.cp`), `for lo in (0..n).step(B): let hi = min(n, lo + B)` is 1 explicit line that already exposes both block bounds and block id without introducing a chunk iterator type.
2. **`D[..k]` — DELETE.** **0 direct uses** in 642 ports; motivated by 1 file (`proto/ports/scout_batch_37/ioi12p5.cp:23, 44`: `for c in Colour if c.to_idx < k:`). Worse, `D[..k]` overloads indexing brackets `[...]` directly onto a **domain/type name**, visually colliding with array slicing `a[..k]`. In the 1 file that wants the first `k` elements of `Colour`, `for c in Colour if c.to_idx < k:` (or `for i in 0..k: let c = Colour.from_idx(i)!`) already reads cleanly with zero new syntax.
3. **`r1 & r2` (range intersection) — DELETE.** **0 direct uses** in 642 ports; motivated by **1 single line** (`proto/ports/scout_batch_32/aac5p3.cp:26`):
   - Before (already clean, 1 line): `route(kid.v, u, max(cuts[i], part.lo)..min(cuts[i + 1], part.hi))`
   - With `r1 & r2`: `route(kid.v, u, (cuts[i]..cuts[i + 1]) & part)` — saves only 16 characters in 1 file out of 642 while overloading bitwise/set `&` onto ranges (requiring extra parens because `&` binds tighter than `..`!).
4. **`.mirror` on domains and ranges — DELETE.** **1 file / 4 occurrences** in 642 ports (`proto/ports/scout_batch_3/coci06c1p6.cp`). Its other cited motivation (`proto/ports/scout_batch_32/noimps23p1.cp:38–41`) operates on `Int` positions `1..=n`, not a nominal domain, so `fn flip(x: Int) -> Int = n + 1 - x` is already a 1-line local helper that works on `Int` and domains alike.

---

### 2.8 `.flipped_rows` / `.flipped_cols` (and `.flipped`) — **DELETE** (keep `.transposed` only)

- **Data:** `.flipped_rows`: **0 files**; `.flipped_cols`: **0 files**; retired `.flipped`: **2 files / 3 occurrences** (`ccc18s2.cp`, `apio09p1.cp`); `.transposed`: **4 files / 6 occurrences**.
- **Mental-overhead problem:** Even after splitting `.flipped` into `.flipped_rows` and `.flipped_cols`, contestants still pause at 2 AM asking *"does `flipped_rows` reverse the order of the rows (vertical flip), or reverse the elements inside each row (horizontal flip)?"* Both English readings are plausible, and picking the wrong one silently rotates 90° CCW instead of CW!

Before (`proto/ports/scout_batch_47/ccc18s2.cp:23`, ambiguous flip name, 30 chars):
```cp
g = g.transposed.flipped_cols
```

After (explicit 2D comprehension — **zero ambiguity**, +17 chars, works in the 2 files that rotate grids):
```cp
g = [N, N] (r, c) => g[N.from_idx(n - 1 - c.to_idx)!, r]
```
(Or with an `Int`-indexed `0..n` grid: `g = [0..n, 0..n] (r, c) => g[n - 1 - c, r]` — 43 chars and impossible to misread!)

**Verdict: DELETE `.flipped_rows`, `.flipped_cols`, and `.flipped`. KEEP `.transposed`** (4 files, mathematically unambiguous).

---

### 2.9 `.absorb()` vs `.take()` — **DELETE `.absorb()`, KEEP `.take()`**

- **Data:** `.absorb()` appears in **0 of 642 canonical ports**; `.take()` appears in **7 files / 8 occurrences**. On the demand side, `.len`-swap + `.clear()` small-to-large merges appear in only **1 canonical port** (`proto/ports/scout_batch_36/cco22p5.cp:32–39`) — and look at what `cco22p5.cp` actually does inside the merge:

```cp
// proto/ports/scout_batch_36/cco22p5.cp:32–39
if self.members[u].len > self.members[v].len: (u, v) = (v, u)
self.pairs -= c2(self.members[u].len) + c2(self.members[v].len)
var moved: List<V> = []
for x in self.members[u]:
    self.rep[x] = v
    self.members[v].push(x)
    moved.push(x)
self.members[u].clear()
```

Plain `dst.absorb(mut src)` **cannot even express `cco22p5.cp`** because each moved vertex must simultaneously update `self.rep[x] = v` and append to `moved`! Meanwhile, v2.1's `.absorb()` has **three incompatible signatures** across containers (`List.absorb(mut other)`, `Set.absorb(mut other)`, `Map.absorb(mut other, combine)`) plus hidden O(1) swap-if-smaller semantics that surprise readers expecting `dst` to stay the receiver.

With `.take()` alone (already needed in 7 ports to move a container out of `[V]List<T>` in O(1)), small-to-large merge on *any* container is 2 uniform lines:

```cp
if dst.len < src.len: swap(mut dst, mut src)
for (k, v) in src.take(): dst[k] = (dst.get(k) ?? 0) + v
```

**Verdict: DELETE `.absorb()`. KEEP `.take()`.**

---

### 2.10 `topo::<V>` and `topo_prefix` — **MERGE `topo::<V>` into inference, DELETE `topo_prefix`**

- **Data:** `topo::<` appears in **1 file** (`proto/ports/scout_batch_27/noi10p4.cp:34`), bare `topo(` in **2 files**, `topo_prefix` in **0 files**.
- Look at why `noi10p4.cp:34` had to write turbofish at all:

Before (`proto/ports/scout_batch_27/noi10p4.cp:18, 34`, 56 chars on line 34 + wrapper tuple `(to: Plane)`):
```cp
var after: [Plane]List<(to: Plane)> = []
let order = topo::<Plane, (to: Plane)>(p => after[p])!
```

After (passing `after: [Plane]List<Plane>` directly under `Neighbours<V = Plane>` + bare `V: Target`, **24 chars, -32 chars, zero turbofish, zero lambda, zero wrapper tuple**):
```cp
var after: [Plane]List<Plane> = []
let order = topo(after)!
```

Teaching `topo::<V>` on the cheat sheet treats a missing type-inference rule as user-facing syntax. Fix inference on `[V]List<E>` and `u => adj[u]` so `topo(adj)` never needs `::<V>`, and delete `topo_prefix` (0 uses; Kahn's loop is 8 lines when a problem wants the non-cyclic prefix).

**Verdict: KEEP `topo(g)`, DELETE `topo::<V>` from docs/cheat sheet, DELETE `topo_prefix` from `std`.**

---

### 2.11 Over-Specialized Search, Sequence, and Stdlib Helpers — **DELETE / DEMOTE TO `examples/local-lib/`**

Five helpers on the v2/v2.1 cheat sheet or `stdlib/seq.cp` have 0–1 uses across 642 ports and are strictly cleaner without them:

1. **`bsearch_first_some` — DELETE.** **1 file / 1 occurrence** (`proto/ports/scout_batch_32/996warmup8p3.cp:45–49`). Look at the real code:

Before (`proto/ports/scout_batch_32/996warmup8p3.cp:45–49`, 4 lines / 145 chars, copies a 10^5-element `List<Ball>` on every `true` midpoint):
```cp
fn first_done(t: Int) -> ?State:
    let s = after(t)
    if any(done(s.hits)): s else: none
let (t1, (hits, balls)) = bsearch_first_some(0..=4e18, first_done)!
```

After (plain `bsearch_first`, **2 lines / 90 chars, -55 chars, zero helper fn, faster because `balls` is not copied during the 62 binary-search probes**):
```cp
let t1 = bsearch_first(0..=4e18, t => any(done(after(t).hits)))!
let (hits, balls) = after(t1)
```

2. **`slide_until` — DELETE.** **1 file / 1 occurrence** (`proto/ports/scout_batch_49/ioi99p6.cp:21`). Stateless `slide_until(r, ok)` cannot even express the 99% of two-pointer problems that maintain incremental state (frequency maps, running sums), and forces contestants to remember whether `ok(l, r)` is inclusive or half-open. In `ioi99p6.cp:21`, a standard 3-line two-pointer loop is completely explicit:

Before (`proto/ports/scout_batch_49/ioi99p6.cp:21`):
```cp
for (top, bottom) in slide_until(Row, (t, b) => peak.fold(t..=b) - pit.fold(t..=b) <= k):
    let area = (bottom - top + 1) * (right - left + 1)
```

After (zero mystery about inclusive vs empty windows):
```cp
var bottom = Row.first
for top in Row:
    while let nb = bottom.next and peak.fold(top..=nb) - pit.fold(top..=nb) <= k:
        bottom = nb
    let area = (bottom - top + 1) * (right - left + 1)
```

3. **`rotate_right` — DELETE (and let `rotate_left` accept signed Euclidean shifts, or merge into `rotate(mut a, k)`).** `rotate_right` has **0 files**; `rotate_left` has **1 file** (`proto/ports/scout_batch_39/aac4p6.cp:33`), which already wrote `rotate_left(mut a, (-k) % n)`. Accepting any signed `k` via Euclidean `k % a.len` eliminates `rotate_right` completely.
4. **`prod(gen)` — DELETE.** **0 files** in 642 canonical ports (integer products overflow `Int` almost immediately unless modular, where a 1-line loop over `Mod<P>` is clearer). Keep `sum(gen)` (119 files / 188 occurrences).
5. **`sliding_min` / `sliding_max` — DELETE** (0 files in 642 ports).
6. **`SparseTable.fold(r) -> ?T` — REVERT to `T` (assert non-empty).** Changing `SparseTable.fold` to return `?T` in v2.1 U10 (and `stdlib/sparse.cp:21` in `9031168`) directly breaks `proto/ports/scout_batch_49/ioi99p6.cp:21` (`peak.fold(t..=b) - pit.fold(t..=b) <= k`), because U1 rule 4 makes `<=` on `?Int` a compile error! Unlike `SegTree` (which has a monoid identity `id` for empty ranges), a `SparseTable` requires an idempotent operation and is queried on non-empty inclusive ranges `l..=r` nearly 100% of the time. Reverting `SparseTable.fold(r)` to return `T` (asserting `not r.is_empty`) eliminates `.fold(l..=r)!` noise on every call while still trapping empty queries safely.
7. **Other `std` trimmings (0–1 uses across 642 ports):**
   - **Delete `Dsu.count`** (duplicate alias of `Dsu.components` in `stdlib/dsu.cp:65`).
   - **Delete `@memo(flat)`** (0 files; redundant because `@memo` already chooses flat tables automatically whenever domain product <= 4×10^7; keep `@memo(hash)` — 3 files — as the sole sparse override).
   - **Delete `floor_div`** (0 files; redundant with CP Lang's built-in Euclidean `/` for positive divisors; keep `ceil_div` — 3 files).
   - **Demote 0–1-use specialized structures from the cheat sheet / core `std` to `examples/local-lib/`:** `MonoDeque` (1), `Steps` (1), `TwoSat` (0), `Matrix` (0), `palindromes` (0), `shortest` (0), `walk_back` (0), `mst` (0), `next_permutation` (0), `factorize` (0), `convex_hull` (0), `Dir8` (1), `MinCostFlow` (1). Keeping them in `examples/local-lib/` (not `std` and not on the one-page cheat sheet) preserves zero-cost availability under Goal G8 without bloating what contestants must memorize.

---

### 2.12 Traits & Decorators Trim

- **Internalize `Copy` into the compiler (8 core traits -> 7).** `T: Copy` appears **0 times** in 642 canonical ports and **0 times** in `stdlib/`. Just as Audit #1 internalized `Small`, `Relocate`, `Niche`, and `Zero` into compiler layout rules, value-copy vs. move-with-`.copy()` is a compiler semantic property that contestants never name as a trait bound.
- **Collapse library-author traits from 4 (`Indexed`, `IndexedMut`, `Iterator`, `Gen`) to 2 (`Indexed`, `Iterable`).** `Indexed` and `IndexedMut` unify under `lends` / `lends mut` indexing; `Gen` needs no trait because choice-stream shrinking works on any `@gen` function taking `rng: Rng`. (Meanwhile `stdlib/graph.cp`'s `Neighbours`/`Target` and `stdlib/algebra.cp`'s `Reversible` remain internal stdlib hooks, not language traits.)
- **Keep all 8 real contestant decorators (`@memo`, `@multitest`, `@brute`, `@tags`, `@gen`, `@checker`, `@interactor`, `@grader`),** noting honestly that v2.1 counted `@checker`/`@interactor` as one slot. Every one of `@memo` (45), `@brute` (26), `@multitest` (24), `@grader` (18), `@tags` (11), `@checker` (8), and `@interactor` (7) proved its worth in the corpus, while `@gen` backs P0 stress-testing. Library-author decorators become 3: `@get` (44 uses across 12 `stdlib/` files), `@test` (5 ports), and `@law` (replacing hard keyword `law`).

---

## 3. Master Keep / Merge / Delete Summary

**KEEP (earned their keep across 642 ports):**
- Nominal `domain`, `[D]T`, `[R, C]T`, ASCII-only `Str`, `.l`/`.r` boundaries, open ranges `..i`/`i..`, `prefix(a)`
- Algebraic `Index` (`Bool`, `enum`, `?T`, bounded ranges, tuples, `Subset<D>`) and nested/block-scoped `@memo` (with auto flat-vs-hash and optional `@memo(hash)`)
- Core Tropical Optionals: `none` as `min`/`max`/`min=`/`max=` identity, `none` as `+`/`-`/unary `-` absorber on `?Int`/`?F64`, `==`/`!=` auto-lifting `T` to `?T`, and compile error on unparenthesized `??` + arithmetic
- `lends` / `lends mut` lexical borrowing + compiler disjointness proofs (auto-derived for simple forwarding, `@law disjoint` for custom views) + `.take()` + `.copy()` + `unchecked` + `a.raw[i]`
- Unified `Mod<P>` (const + runtime), `Rooted<V>` (`sub`, `chains`, `lca`, `dist`, `lift_while`), `SegTree`, `LazySegTree`, `Fenwick`, `Dsu` (`components`), `Graph`, `Bitset`, `MinHeap`/`MaxHeap`, `Deque`, `Set`/`Map`/`HashSet`/`HashMap`/`SortedBag`, `FlowGraph`, `Dir4`, `Pt`, `Sieve`, `compress`, `scan`, `runs`, `sum`, `bsearch_first`/`bsearch_last`, `first_ge`/`last_le`, `min_by`/`max_by`, `.transposed`
- `@grader` on `fn` and `struct`, `@multitest`, `@tags`, `@brute`, `@gen`, `@checker`, `@interactor`, `@get`, `@test`
- Four numeric types: `Int`, `U64`, `I128`, `F64`

**MERGE / SIMPLIFY:**
- `lend` -> merge into tail expression / `return` inside `lends` bodies
- `law` (hard keyword) -> demote to `@law` library-author decorator + auto-derive forwarding `disjoint`
- `topo::<V>(u => adj[u])` -> merge into type inference (`topo(adj)` with zero turbofish)
- `rotate_left` + `rotate_right` -> single `rotate_left(mut a, k)` accepting signed Euclidean `k` (or `rotate(mut a, k)`)
- `Indexed` + `IndexedMut` -> single `Indexed` library-author trait
- `Copy` -> internalize into compiler layout/move semantics (not a user-visible trait)
- `SparseTable.fold(r) -> ?T` -> revert to `T` (assert non-empty, unbreaking `peak.fold(t..=b) - pit.fold(t..=b) <= k`)

**DELETE (from core language, `std`, and `docs/one-page-cheat-sheet.md`):**
- Keywords: `defer`, `where`, `inline`, `lend` (plus demoting `law` to `@law`, and cleaning zombie `with`, `by`, `<=>` from `token.rs`)
- Syntax & sub-rules: `for mut x in xs:`, `@grader const`, uninitialized file-scope grader domains, `@memo(flat)`, `?T` absorption on `sum(gen)`, `+=`/`-=` lifting on `?T`
- Domain/range/grid methods: `D.chunks(k)`, `D[..k]`, `r1 & r2`, `.mirror`, `.flipped_rows`, `.flipped_cols`, `.flipped`
- Container/search/seq helpers: `.absorb()`, `Marks<D>` (move to `local-lib` at most), `bsearch_first_some`, `slide_until`, `rotate_right`, `prod(gen)`, `sliding_min`/`sliding_max`, `topo_prefix`, `Dsu.count`, `floor_div`
- Demote to `examples/local-lib/` (out of core `std` & cheat sheet): `MonoDeque`, `Steps`, `TwoSat`, `Matrix`, `palindromes`, `shortest`, `walk_back`, `mst`, `next_permutation`, `factorize`, `convex_hull`, `Dir8`, `MinCostFlow`, `Gen` trait

---

## 4. Updated Concept-Budget Count (v2.1 vs. Post-Audit #2)

- **Reserved keywords:** **40 -> 35 total** (target `<= 40`, comfortably met with 5 keywords of headroom).
  - **31 contestant keywords** (down from 32, deleting `defer`): `let`, `var`, `const`, `fn`, `struct`, `enum`, `domain`, `type`, `use`, `pub`, `if`, `elif`, `else`, `for`, `while`, `in`, `match`, `return`, `break`, `continue`, `and`, `or`, `not`, `mut`, `self`, `true`, `false`, `none`, `assert`, `assume`, `unchecked`. Single-file contest solutions typically use ~25.
  - **4 library-author keywords** (down from 8, deleting `where`, `inline`, `lend`, and demoting `law` to `@law`): `trait`, `impl`, `lends`, `yield`.
- **Numeric types:** **4** (`Int`, `U64`, `I128`, `F64`) — unchanged.
- **Core traits:** **7 auto-derived core traits** (down from 8, target `<= 8`, met): `Index`, `Seq`, `Eq`, `Ord`, `Hash`, `Show`, `Read` (`Copy` internalized). Plus **2 library-author traits** (`Indexed`, `Iterable`, down from 4).
- **`@` decorators:** **8 contestant** (`@memo`, `@multitest`, `@brute`, `@tags`, `@gen`, `@checker`, `@interactor`, `@grader` — counted honestly without collapsing `@checker`/`@interactor`) + **3 library-author** (`@get`, `@test`, `@law`).
- **Search names:** **7 -> 6** (`bsearch_first`, `bsearch_last`, `first_ge`, `last_le`, `min_by`/`max_by`, `find`/`first` — deleting `bsearch_first_some` and `slide_until`, plus cleaning legacy `first_where` in older ports).
- **One-page cheat sheet (`docs/one-page-cheat-sheet.md`):** currently **242 lines (20.3 KB)**; removing the deleted v2.1 entries recovers ~35 lines, returning the reference to **~205 lines** — cleanly fitting on one dense printable page.
