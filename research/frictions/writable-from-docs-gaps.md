# Writable-from-docs language & stdlib gaps

Observed while evaluating 55 unseen mixed-difficulty contest statements (`research/writable-from-docs-set.json`) with `gpt-5.4` given only the CP Lang docs bundle. These are genuine language/stdlib frictions (not docs gaps) routed to their owning lanes.

## 1. `bsearch_last(0..n, ...)` return type fails to unify with `?Int`

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
