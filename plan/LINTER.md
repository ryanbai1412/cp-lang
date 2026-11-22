# CP Lang linter: architecture and design

Draft 1. Companion to `spec/idioms/principles.md` (why) and `spec/idioms/lints.md` (what).

## 1. What the linter is for

- Catch legal code that is probably **wrong, slow or unidiomatic**, and say what to write instead.
- Be the main way people learn the language: every finding names the practice it serves, shows the rewrite, and links a page.
- Never block a build. Levels decide whether `cpl lint` exits non-zero; `cpl check`, `run` and `build` ignore lints.

## 2. Where it stands today (measured on the v0.2 deletion branch)

| | Today | Problem |
|---|---|---|
| Size | `cpl-lint`: 14k lines, 38 lints; `rules/taste.rs` alone is 4.5k | One big file per theme, no per-lint unit |
| Input | Syntax tree only (`cpl-syntax`), one walk per rule | No types, no name resolution, so many lints guess |
| Analysis | Its own range and bound analysis (`bounds.rs`, `rules/mod.rs` "proven" facts, `cpl-prover`) | Duplicates what the checker knows; the LSP reads its inlay hints from the **linter** |
| Metadata | Rust `const Lint {..}` per lint, with trigger and fix strings inline | Docs, skill and site can't be generated from it without parsing Rust |
| Stdlib names | Hard-coded in rules (`"Prefix"`, `"sorted"`, `"bsearch_first"`, ...) | Violates the spec: a lint is declared by the library that owns the names; some already name removed APIs |
| Config | `cp.toml [lints]`, `@allow(name)` | Spec says `cpl.toml` and `with lint(name: allow)` |
| Browser | Semantic half compiled out on wasm (`cpl-check` is a non-wasm dependency) | Playground shows fewer lints than the CLI |
| Tests | Unit tests in `tests.rs` (2.1k lines) | No near-miss tests, no "fix keeps behaviour" test, no false-positive ratchet |

## 3. Target architecture

```
source ──parse──▶ syntax tree ──check──▶ typed IR + facts ──▶ lint engine ──▶ findings + fixes
                      │                        │                   ▲
                      └──── syntactic lints ───┘                   │
                                                lint records (data) ┘ ◀── stdlib / user libraries declare rewrites
```

### 3.1 Two kinds of lint, one engine
- **Syntactic lints** run on the syntax tree. Spelling and formatter-adjacent rules (`getter_parens`, `redundant_arg_name`, `turbofish_type_args`). They work on code that doesn't type-check, so the editor gets them while you type.
- **Semantic lints** run on the checked, typed IR, the same IR the interpreter and C++ lowering use. Names are resolved, every expression has a type, every array knows its domain, every loop knows its range.
- One walk per file dispatches each node to every lint registered for that node kind, instead of 38 separate walks. This keeps lint time linear and inside the 500 ms front-end budget.

### 3.2 Facts come from the checker, not the linter
The checker already proves widths, index safety and overflow. Those proofs become a **fact table** owned by `cpl-check`:
- the value range of each expression and variable, from declared bounds;
- whether each index is proven in range;
- the size of each domain and array, and the trip count of each loop;
- the cost class of each call, from the stdlib's documented costs.

The linter, the LSP's inlay hints and `cpl explain` all read the same table. `bounds.rs`, the "proven" passes and the linter's use of `cpl-prover` move into the checker or are deleted. That removes about 3k duplicated lines and makes bound-aware lints exact: "this loop nest is about 4e10 steps at the declared `n`" uses the same numbers the checker uses for widths.

### 3.3 Shape recognizers
Most of the ~670 catalogued lints are "this loop is really algorithm X". Instead of one analysis each, a small set of **recognizers** classifies loops and helpers once per file:

| Recognizer | Matches | Feeds lints like |
|---|---|---|
| fold | accumulator updated once per iteration with an associative op | `loop_reduction`, `free_fold`, sum/min/max/count |
| search | loop that breaks or returns on the first hit | `search_loop`, `find`, `any`, `all`, `position` |
| build | push or assign `out[i] = f(i)` into a fresh container | `push_loop`, comprehension, `[D] i => ...` |
| scan | `p[i + 1] = p[i] op a[i]` | `hand_rolled_prefix_sum`, `scan`, `Prefix` |
| bisect | `lo`/`hi` narrowing on a monotone predicate | `hand_rolled_answer_search`, `first_true` |
| two pointers / window | two indices that only advance | `windows`, `two_pointers` |
| union-find | parent array plus find-with-compression | `hand_rolled_dsu` |
| sieve | composite marking up to a bound | `hand_rolled_sieve` |
| lifting | `up[k][v] = up[k-1][up[k-1][v]]` | `hand_rolled_binary_lifting` |
| copy-then-writeback | copy a range, transform, write back | `copy_sort_writeback`, in-place slice ops |
| sentinel | a value used as "absent" (`-1`, `INF`, `n`) | optional lints |
| index-space | one integer indexing arrays over different domains | domain lints |

A recognizer outputs a typed **shape** (for a fold: the accumulator, the operator, the identity, the range, the element expression). Lints match shapes, not raw trees, so `for i in 0..n`, `for i in D` and `while i < n: ... i += 1` all match the same lint.

### 3.4 Libraries declare their own lints
Per the spec, the linter has no hard-coded stdlib names. A library declares a rewrite next to the API it recommends:

```text
/// Prefix sums over a sequence.
@lint(hand_rolled_prefix_sum, from: scan(op: +, over: a), to: "Prefix({a})", tier: fix)
pub fn Prefix<D, T: AddGroup>(a: [D]T) -> Prefix<D, T>: ...
```

- `from:` is a recognizer shape, optionally with constraints such as `op: +` or `elem: Int`. Simple cases use a code pattern with holes instead.
- `to:` is the replacement, with holes filled from the match.
- User libraries get exactly the same mechanism. A personal library can ship "you hand-rolled my `Hld`" lints, which is Goal 6.
- The engine checks the replacement type-checks before offering it. If it doesn't, the lint downgrades to a warning with no fix, so a lint can never offer a broken autofix.

This needs one language feature: a general `@lint` attribute on items, read by the checker. Decorators are already general (`@memo`, `@get`, `@alt`), so this is an instance of the existing mechanism, not a special case.

### 3.5 One record per lint, everything generated from it
Each lint is one directory, `spec/lints/<name>/`:

```
lint.toml        code, name, family, tier, default level, practice, one-line message
before.cp        fires here
after.cp         the fix; same output as before.cp
near_miss.cp     must not fire (the case people would expect a false positive on)
README.md        why, with prose for the docs page
```

Generated from the records:
- the Rust registry, so a lint's code, name and level can't drift from its docs;
- `spec/diagnostics.md` lint codes;
- the site's per-lint pages and the idioms guide;
- the skill's rewrite table and common-mistakes list;
- `cpl explain L0412`.

Stdlib-declared lints (3.4) carry the same fields in their `@lint` attribute, plus test files beside the module.

## 4. Fixes

- A fix is a structured edit on the typed IR, printed by the formatter, so every fix comes out canonically formatted.
- **Tiers:** `fix` is applied by `cpl lint --fix`; `fix?` needs `--fix=review` or an editor click; `warn` has no fix; `doc` is documentation only and never fires.
- **Every fix is verified, not trusted:**
  1. The fixed file must pass `cpl check`, or the fix is dropped.
  2. In the test suite, `before.cp` and `after.cp` must print the same output in the interpreter and in the C++.
  3. For `fix?` tiers, the suite also stress-tests before against after with the module's `@gen`, when it has one.
- Fixes that change cost (a hand-written loop becoming a library call) say the cost in the message, such as "O(n) once, then O(1) per query".

## 5. Messages are the teaching surface

```text
warning[L0610] hand_rolled_prefix_sum: this loop builds prefix sums of `a`
  --> sol.cp:7:1
   |
 7 | var p = [0] * (n + 1)
 8 | for i in 0..n:
 9 |     p[i + 1] = p[i] + a[i]
   |
   = practice 1, name the algorithm: `Prefix(a)` says it, and `p.fold(l..r)` answers range sums
   = fix: let p = Prefix(a)
   = learn: cp-lang.org/lints/hand_rolled_prefix_sum
```

- One finding per root cause. When three lints match the same loop, the most specific one wins and the others are suppressed.
- `cpl lint --teach` prints the before/after diff for each finding, for people learning from their own code.
- The playground shows the same findings with a one-click "apply" button.

## 6. Levels and configuration

- **Groups,** like Clippy's: `correctness` (deny), `perf` (warn), `idiom` (warn), `style` (warn), `pedantic` (allow).
- **Project level:** `cpl.toml [lints]`, by group or by name.
- **In code:** `with lint(name: allow)` per the spec. `@allow(name)` is removed, so there is one spelling.
- `cpl lint --level=pedantic` turns on everything for a review pass.

## 7. Surfaces

All of these use the same engine and the same findings:
- CLI: `cpl lint`, `--fix`, `--teach`, `--json`.
- LSP: diagnostics, quick-fix code actions, and hover links to the lint page.
- The playground. This needs the checker in wasm, which removes the current split where the browser has fewer lints.
- `cpl stress` and judge reports, so a failing run can point at a suspicious lint ("this `==` on floats is in the failing branch").
- The skill eval: lint findings on agent-written code are the idiom-quality score.

## 8. Testing

1. **Per-lint records** (3.5): fires on `before.cp`, silent on `near_miss.cp`, and `--fix` turns `before.cp` into `after.cp` byte for byte. `before.cp` and `after.cp` agree in the interpreter and in the C++.
2. **False-positive ratchet:** lint every `corpus/problems/*/spec.cp` and every file in `spec/std`. These are idiomatic by definition, so each finding is either a false positive or a port to fix. The count only goes down.
3. **Self-lint:** the stdlib and site snippets are lint-clean at the default level, enforced in `check.sh`.
4. **Fix fuzzing:** apply every available fix to the corpus ports and to the pending v0.2-era programs, and require them to still check and still print the same output.
5. **Budget:** linting the largest corpus file takes under 50 ms, enforced by a test.

## 8a. False positives

A wrong warning costs more than a missed one, so:

1. **Before, after, near miss.** Every lint record has all three; a lint cannot land without them. Each reported false positive becomes a permanent near-miss case.
2. **Silent when unsure.** Recognizers fire only on proven shapes (accumulator not read elsewhere, no early exit, operator matches). An uncertain analysis returns nothing.
3. **Corpus gate.** Every lint runs over the corpus and spec examples; each hit is classified true/false. A lint starts opt-in and becomes default-on only after a run with zero false positives. CI keeps re-running the gate, so a regression demotes it.
4. **Proven fixes.** `fix` must re-check and agree in interpreter and C++ (stress for `fix?`); otherwise the lint becomes a suggestion with no autofix.
5. **Clear beats short.** Shortening lints stay silent when the replacement is harder to read.
6. **Scoped silencing with a reason**, `with lint(name: allow)`; lints silenced often are flagged for redesign.
7. **Teach mode is separate.** Suggestion-only lints never warn; they appear under `cpl lint --teach` and in docs.

Catalogue tiers today: 262 `fix`, 154 `fix?`, 242 `warn`, 15 `doc`; all start opt-in.

## 9. Migration from today, in order

1. Move lint metadata into `spec/lints/<name>/` records and generate the Rust registry, with no behaviour change. The 38 lints gain near-miss and fix tests.
2. Move the bound facts into `cpl-check` as a fact table; the LSP and the linter read it. Delete the linter's own analysis.
3. Run the semantic lints on the typed IR, using the one-walk dispatcher.
4. Build the recognizers and port the existing `hand_rolled_*`, `loop_reduction` and `search_loop` lints onto them.
5. Add `@lint` declarations in the stdlib and delete the hard-coded stdlib names from the linter.
6. Rename config to `cpl.toml` and `with lint(...)`.
7. Compile the checker for wasm so the playground gets every lint.
8. Then grow the catalogue family by family from `spec/idioms/lints.md` (about 670 entries), most-valuable first: bound-aware correctness, P0 bug classes, then stdlib replacements.

Steps 1–5 are architecture work, about a day. After that, a typical new lint is a record plus a `@lint` line and needs no Rust.

## 10. Decided (Ryan, 2026-11-22)

- `@lint` attribute next to the API it recommends: yes.
- Idiom lints warn by default, once each passes the corpus gate (8a).
