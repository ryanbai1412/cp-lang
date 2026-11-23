# Idiomatic code checklist

| | |
|---|---|
| Kind | Checklist |
| Owner | idioms |
| Explains | `handbook/guidelines/idiomatic-code.md` |

Every rule a port, dream, skill page or docs example must follow, each with
a stable id that reviews, lints and tests cite. The ideas behind them are in
`handbook/guidelines/idiomatic-code.md`.

# Reference

### idiomatic-code/R1

**Rule:** the header has `Source:`, `Problem:` and `Solution:`, and derives every constant and closing formula the code uses.
**Why:** a reader learns the idea before the code, and nothing is unexplained.
**Example:** `// Solution: free cuts split the items into SCCs ...` before `@!multitest`.
**Enforced by:** `corpus/header.py` (reports); review.

### idiomatic-code/R2

**Rule:** the program reads like the editorial; keep the C++ construction only where the editorial uses it.
**Why:** the invariant or recurrence is what a reader checks; index bookkeeping hides it.
**Example:** a named `SegTree` with range add, not a hand-indexed array of nodes.
**Enforced by:** cold taste review, ratcheted by `scripts/review-check.py`.

### idiomatic-code/R3

**Rule:** use std or the language for a standard structure or algorithm; a hand-rolled one needs a friction with a run.
**Why:** a hand-rolled copy is longer, can be wrong, and hides what std lacks.
**Example:** `xs.first(p)`, not a loop setting a `found` flag.
**Enforced by:** the hand-rolled-std lint family; `crates/cpl-lint/src/corpus_tests.rs` (`corpus_warnings_only_go_down`).

### idiomatic-code/R4

**Rule:** each fact is written once: a bound, base, modulus, loop or formula.
**Why:** two copies drift apart, and a change has to find both.
**Example:** `domain I = 1..=n` once, not `n` and `n + 1` across the file.
**Enforced by:** review.

### idiomatic-code/R5

**Rule:** every statement bound today's types can hold is a type; absent values are `?T`, never a sentinel.
**Why:** the type validates input, picks storage width and proves overflow away.
**Example:** `let x: 1..=10_000_000 = read()`, not `let x: Int = read()`; `?I`, not `-1`.
**Enforced by:** review; a check of read types against `meta.json` bounds: `missing`.

### idiomatic-code/R6

**Rule:** distinct index spaces are distinct domains and never cross through a raw `Int`.
**Why:** mixing vertices with edges or positions with cuts is a P0 bug class the types can rule out.
**Example:** `domain Item = 1..=n` and `domain Cut = 0..=n`.
**Enforced by:** review.

### idiomatic-code/R7

**Rule:** no side effect inside an expression, no discarded result, no state threaded by hand where a closure can own it, and no state carried from one test to the next.
**Why:** hidden effects make the code lie about what it does.
**Example:** not `first(xs, x => try_add(x))`; not `let _ = f()`.
**Enforced by:** review.

### idiomatic-code/R8

**Rule:** use the spec's one spelling per idea.
**Why:** a reader or model learns one form; a second spelling teaches a removed one.
**Example:** `@!multitest`, `Some(x)`, `None`, `Mod<P>`, `.items`, `xs.sum()`; free form over a generator (`sum(f(x) for x in xs)`) and two values (`min(a, b)`).
**Enforced by:** `cpl check` errors on removed spellings (E0503), so `scripts/skill-gen.py --check` covers the skill's code blocks; `corpus/differential.py` (`removed-spelling`) covers every v03 port, dream and spec, which need not check; `scripts/skill-prose.py` covers inline code in the skill's prose; prose in the other docs: `missing`.

### idiomatic-code/R9

**Rule:** `cpl check` and `cpl lint` are clean, and corpus warning counts don't go up.
**Why:** published code is what lints are tuned on; a warning there is a bug in the code or the lint.
**Example:** a wrong lint goes to lint-fmt, not into an `allow`.
**Enforced by:** commit-tier `cpl check` and lint ratchet on staged ports; `scripts/port-check.sh`.

### idiomatic-code/R10

**Rule:** a port runs within 1.1x of its matched C++ on a max input, or has a performance friction with the measurement.
**Why:** a slow idiom is a design bug; the obvious way must be the fast way.
**Example:** see `handbook/guidelines/performance.md` for how to measure.
**Enforced by:** perf's corpus bench; review.

### idiomatic-code/R11

**Rule:** a friction claiming the language lacks something quotes the dream line run under a freshly built `cpl` and its error.
**Why:** unverified gaps send other domains after problems that are already solved.
**Example:** `cpl run` output with `error[E0504]` pasted in `research/frictions/<id>.md`.
**Enforced by:** review.

### idiomatic-code/R12

**Rule:** every operation's inputs are in its domain, by type or by a check the reader can see.
**Why:** a zero divisor, an overflowing product, a too-wide shift or a `!` on an absent value passes the samples and fails the system tests.
**Example:** `n.isqrt`, not `floor(sqrt(n))`; floats compare with a tolerance; `xs.max()!` only after `if xs.len > 0:`.
**Enforced by:** the correctness lints citing it in `spec/lints/<name>/lint.toml`.

### idiomatic-code/R13

**Rule:** every line has an effect the reader can see.
**Why:** unreachable code, unused bindings, checks that always pass and silent shadowing are where a revision goes wrong.
**Example:** `let` for a binding never reassigned, not `var`; no `with lint(...)` naming no lint.
**Enforced by:** the style lints citing it in `spec/lints/<name>/lint.toml`.
