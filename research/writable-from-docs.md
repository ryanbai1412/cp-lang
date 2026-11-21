# Writable-from-docs benchmark & improvement curve

Pinned benchmark set: `research/writable-from-docs-set.json` (**55 unseen mixed-difficulty problems**: 35 Codeforces Div2 B–E, 12 AtCoder ABC D–E, 8 AtCoder ARC A–B; zero overlap with `proto/ports/`).

Model: `gpt-5.4` given **only** the CP Lang documentation bundle (`docs/one-page-cheat-sheet.md`, `site/content/tour.md`, `types-and-control.md`, `io-and-formatting.md`, `indices.md`, `borrowing.md`, `strings.md`, `stdlib.md`, `cpp-guide.md`, `idioms.md`) and one problem statement at a time.

Ratchet file: `research/writable-from-docs.json` (enforced in `scripts/scoreboard.py --fast --check` and weekly in `.github/workflows/writable-from-docs.yml`).

## Improvement curve (55-problem mixed-difficulty set, `gpt-5.4`)

| Round | Label | Docs changes applied before run | `cpl check` + `cpl build` | Full pass (all samples) | Notes |
| --- | --- | --- | --- | --- | --- |
| **0** | `round-0-baseline` | Pre-existing 8-file docs bundle (no `idioms.md`) | **40/55 (72.7%)** | **14/55 (25.4%)** | Main gaps: domain-vs-`Int` (`E0301`/`E0302`/`E0309`, 5×), `read()` on `List<Int>` (`E0501`, 2×), `use` keyword (`E0106`), `Sieve.is_prime` method vs index (`E0305`), named tuples `.0` (`E0203`), 2D vs nested indexing (`E0305`) |
| **1** | `round-1-after-idioms-and-r0-fixes` | Added gold-standard `idioms.md` + upgraded `cpp-guide.md` + 6 Round-0 docs fixes | **46/55 (83.6%)** | **19/55 (34.5%)** | +10.9 pp compile, +9.1 pp full pass. Remaining gaps: inline conditional syntax, multi-line `match =>`, `Dsu<V>` named domain, `v.next: ?V` in ranges |
| **2** | `round-2-after-ternary-and-domain-fixes` | Fixed `Dsu<V>` & `v.next` docs; introduced prose typo `if c then a else b` | 40/55 (72.7%) | 18/55 (32.7%) | Harness immediately caught our own `if ... then` prose typo on 8 problems (`E0103`) |
| **3** | `round-3-after-if-colon-and-swap-fixes` | Fixed `if cond: a else: b` and `arr.swap(i, j)` docs | 38/55 (69.1%) | 17/55 (30.9%) | Surfaced 2 more doc inaccuracies: `Sieve.spf[x]` is a list field (not method `spf(x)`), and `HashMap` `m[k]` auto-defaults to `V` (not `?V`) |
| **4** | `round-4-after-sieve-and-hashmap-fixes` | Fixed `Sieve.spf[x]` field & `HashMap` `m[k]` vs `m.get(k)` docs | **46/55 (83.6%)** | **17/55 (30.9%)** | Plateau confirmed (~83.6% compile, ~31–35% full pass across stochastic GPT-5.4 runs); remaining failures are predominantly algorithmic WAs on Div2 C–E / ARC B |

## Ratchet policy

- Floors are set to `max(prev_floor, observed_rate - 0.05)` (5% noise margin because `gpt-5.4` does not accept `temperature=0`, yielding ~±2-problem variance on `n = 55`).
- Current floors in `research/writable-from-docs.json`:
  - `min_compile_rate`: **78.64%**
  - `min_pass_rate`: **29.55%**
- `scripts/scoreboard.py --fast --check` reads `research/writable-from-docs.json` offline and fails if `latest` ever regresses below either floor; `.github/workflows/writable-from-docs.yml` runs the live benchmark weekly on Monday 06:00 UTC (`workflow_dispatch` supported).

## Remaining docs / compiler gaps observed at plateau

1. `reverse(mut a)` vs `a.reverse()` (`cf2004c`, `cf2001c` in R4: model guessed free `reverse(...)`).
2. Multi-line `match` arm syntax (`cf2007e` still occasionally writes `pat =>\n ...`).
3. Float output formatting precision (`abc382e`: 16-digit sample vs default float formatting).
4. Algorithmic WAs on Div2 C–E / ARC B account for the gap between ~84% compile and ~31–35% full pass.
