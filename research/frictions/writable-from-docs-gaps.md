# Writable-from-docs language & stdlib gaps

Observed while evaluating 55 unseen mixed-difficulty contest statements (`research/writable-from-docs-set.json`) with `gpt-5.4` given only the CP Lang docs bundle. These are genuine language/stdlib frictions (not docs gaps) routed to their owning lanes.

## 1. `bsearch_last(0..n, ...)` return type fails to unify with `?Int`

- **Status:** fixed: `cpl-check` now types `.len` as `Int` (and `.is_empty` as `Bool`) on built-in sequences and widens `0..unknown` ranges to `Range<Int>`, so `bsearch_last(0..blacks.len, ...)` infers `?Int` directly.
- **Severity:** medium
- **Goal it hurts:** express
- **Owning lane:** typechecker / stdlib (`std.search`)
- **Evidence:** On `abc386d` (Round 5), `fn last_black_before_row(...) -> ?Int: bsearch_last(0..blacks.len, i => ...)` failed `cpl check` with `error[E0302]: last_black_before_row returns an ?Int, but bsearch_last(0..blacks.len, ...) is a ?_`.
- **Expected:** `bsearch_last` over a structural `Int` range `0..n` should infer `?Int` directly so returning it from a `-> ?Int` function compiles without extra annotation.

## 2. Multi-statement `match` arms require a helper or `if`/`elif`

- **Severity:** low
- **Goal it hurts:** express
- **Owning lane:** parser / syntax (`crates/cpl-syntax`)
- **Evidence:** Across Rounds 0–5 (`cf2007e`, `abc380e`), models repeatedly wrote indented multi-statement blocks after `pat =>\n    ...`, hitting `E0102: expected a value after =>, found the end of the line`.
- **Expected:** Either support indented block bodies after `pat =>` (matching `if`/`for`) or emit a targeted `E0102` help note pointing to single-expression `pat => expr` / `if`/`elif`.

## 3. `HashMap<K, V>` indexing `m[k]` vs `m.get(k)` optionality surprise

- **Severity:** low
- **Goal it hurts:** hard-to-get-wrong
- **Owning lane:** stdlib / containers (`stdlib/map.cp`)
- **Evidence:** Across Rounds 1–5 (`cf2001d`, `cf2025d`, `abc385d`), models wrote `m[k] ?? 0`, `m[k]!`, or `m[k] ??= ...` expecting `m[k]` to return `?V` (like `List.get`), hitting `E0302`/`E0309` because `m[k]` auto-inserts the default `V`.
- **Expected:** Diagnostics on `m[k] ?? ...` / `m[k]!` could hint `use m.get(k) for an optional lookup, or m[k] directly for default-initialized access`.

## 4. `prev_less`/`next_less` over plain-range arrays leaked internal domain `D`

- **Status:** fixed: `cpl-ir` now applies `plain_ty` coercion to `ExprKind::Index` results, so indexing `[0..n]?D` or `[1..=n]?D` returns `?Int` directly (`prev_less_range.cp` parity test).
- **Severity:** medium
- **Goal it hurts:** express
- **Area:** seq
- **Evidence:** Writable-from-docs seq probe 1 (largest rectangle in histogram) wrote `let l = pl[i] ?? -1; let r = nl[i] ?? n; best max= a[i] * (r - l - 1)` on `a: [0..n]Int` and hit `E0309: '-' can't be applied to D and D`.
- **Expected:** Plain-range arrays `[0..n]T` / `[1..=n]T` passed to generic `[D]T -> [D]?D` helpers should yield `?Int` on indexing, matching `D` position coercion at call boundaries.

## 5. `site/content/stdlib.md` drifted from `std.seq` method names and `runs(by:)` return shape

- **Status:** fixed: updated `site/content/stdlib.md` to match the real surface (`xs.prod()`, `a.runs(by:)` returning `(val, len)` vs `a.chunk_by`/`a.group(by:)`, `rotate_left`/`rotate_right`, `dedup`/`distinct`, `perm_cycles`, and `std.seq` module table entries).
- **Severity:** medium
- **Goal it hurts:** express
- **Area:** seq
- **Evidence:** Writable-from-docs seq probes 2–3 guessed `xs.product()`, `a.rotate(k)`, `a.distinct()`, `a.dedup()`, `cycles(p)`, and `for (k, grp) in items.runs(by: ...)` directly from `site/content/stdlib.md` and failed `cpl check`.
- **Expected:** Every name and return shape in `site/content/stdlib.md` matches the checked stdlib surface.

