# Writable-from-Docs Audit

Automated measurement of the `AGENTS.md` **"Writable from the docs"** rule via `scripts/writable-from-docs.py`.

Each run sends a fresh, zero-context model (`gpt-4o`, temperature `0`) ONLY the 8-file CP Lang docs bundle (`docs/one-page-cheat-sheet.md` + `site/content/{tour,types-and-control,io-and-formatting,indices,borrowing,strings,stdlib}.md`) and one unseen problem statement from `corpus/statements/` (no existing port in `proto/ports/`), then checks the generated `.cp` program with `cpl check`, `cpl build`, and `cpl run` on every sample case.

## Problem set (10 unseen AtCoder statements)

`abc380b`, `abc380c`, `abc381c`, `abc382c`, `abc383b`, `abc383c`, `abc384c`, `abc385c`, `abc386c`, `abc387c` (none have a port in `proto/ports/`).

## Before vs After docs fixes

| Run | Compile pass (`cpl check`) | Full sample + build pass | Distinct language/docs error codes |
| --- | --- | --- | --- |
| Baseline (before docs fixes) | **7/10** | **6/10** | `E0103` (multi-`for` comprehension), `E0203` (`Int.to_str`, `Int.abs`), `E0303` (`read(Int)`), `R0001` (`[1..=n]` indexed from `0`) |
| After docs fixes | **8/10** | **4/10** (plus 1 TLE on `10^18` brute force, 2 algorithm/input bugs) | `E0103` (multi-`for` comprehension in `abc383c`), `E0203` (`List.reserve` in `abc380c`) |

### Docs gaps found and fixed

1. **`Int` to `Str` conversion (`abc387c`, `E0203`):** model called `n.to_str()`. Documented string interpolation `"{n}"` and `std.math.digits(n)` in `site/content/strings.md` and `docs/one-page-cheat-sheet.md`. Result: `abc387c` compiled and passed small samples on re-run.
2. **Typed `read()` and free `abs(x)` (`abc386c`, `E0303` + `E0203`):** model called `read(Int)` and `(len_s - len_t).abs()`. Documented that `read()` always infers from the binding type (`let k: Int = read()`, never `read(Int)`) and that `abs(x)` is a free function in `site/content/io-and-formatting.md`, `site/content/types-and-control.md`, and `docs/one-page-cheat-sheet.md`. Result: `abc386c` went from 2 compile errors to **6/6 PASS**.
3. **1-based array bounds (`abc382c`, `R0001`):** model declared `[1..=m]Int` and indexed with `0..m`. Documented matching loop bounds to declared array bounds in `site/content/indices.md`.
4. **Single-`for` comprehensions & built-in `.adj4` (`abc383c`, `E0103`):** model wrote `sum(1 for r in Row, c in Col if ...)`. Documented single-`for` comprehension scope and built-in `(Row, Col).adj4` in `site/content/types-and-control.md` and `site/content/indices.md`.
5. **`Deque` push method (`abc383c`, `E0203` in intermediate run):** model called `q.push_back(...)`. Clarified `.push`/`.pop` at back (no `.push_back`) in `docs/one-page-cheat-sheet.md`.
6. **Negative-example priming lesson:** writing the literal forbidden token sequence `let mut` inside a code comment caused `gpt-4o` to emit `let mut` on 3/10 problems (`E0106`); phrasing it positively (`use var for mutable variables`) immediately eliminated all `let mut` errors (0/10).

## Latest run per-problem results (`gpt-4o`)

| Problem | Check | Build | Samples | Codes | Outcome / First error |
| --- | --- | --- | --- | --- | --- |
| `abc380b` | ok | ok | 3/3 | — | PASS |
| `abc380c` | fail | — | 0/2 | E0203 | `List<Char>` has no method `reserve` |
| `abc381c` | ok | ok | 2/3 | — | sample 3 WA (algorithm edge case) |
| `abc382c` | ok | ok | 0/3 | — | sample 1 runtime error (input format mismatch in model's code) |
| `abc383b` | ok | ok | 0/3 | — | sample 1 runtime error (input format mismatch in model's code) |
| `abc383c` | fail | — | 0/3 | E0103 | multi-`for` generator comprehension (`for r in Row, c in Col`) |
| `abc384c` | ok | ok | 3/3 | — | PASS |
| `abc385c` | ok | ok | 3/3 | — | PASS |
| `abc386c` | ok | ok | 6/6 | — | PASS |
| `abc387c` | ok | ok | 2/3 | — | sample 3 TLE (`O(R-L)` brute force on `10^18`) |
