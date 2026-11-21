# GOALS: the constitution (every agent reads this; full rationale in ALIGN.md)

**Mission.** Minimize the time it takes a contestant to turn an algorithmic idea into an accepted submission
they understand and trust: quick to express and revise, common mistakes hard to write, failures easy to
explain, with the performance and control of hand-written C++ and a predictable path onto the judge. Judge the
whole contest attempt (drafting, changing your mind, debugging), not only the finished port.

## Goals (each has a test; a claim without a passing test is not a claim)

**I. Express and revise (P0).** Trivial problems take ~3 lines with no ceremony. Hard solutions expose their
invariant, recurrence or algorithm and read like the editorial. Structure is adopted progressively and pays
for itself where introduced (one declaration per semantic index space protects every array over it). A
change of requirement (DP gains a dimension, search returns an end position, array becomes a map) is a local
edit. Graceful off the happy path: every convenience (base conversion, yes/no printing, defaults)
has a local override in one obvious step, so the cold path costs one edit, never a file-wide rewrite. Guessable APIs, natural recursion and closures, cheap algorithm customization, imperative code where the
algorithm is imperative, output as easy as input, single-test and multitest identical. Calm at 3am.
*Test:* the revision test (unseen problem, then one requirement changes: where does the writer hesitate,
duplicate a fact, or rewrite unrelated code?); port reviews leave no awkward spot unfixed. [proposed]
Contestant-written tokens (stdlib excluded) median <= 0.6x the C++ original, none > 1.0x; a requirement
change touches one chunk, median <= 3 lines; trivial problems <= 3 lines; a cold-path variant is one local
edit; unseen-task benchmark median time to AC <= 0.8x C++.

**II. Hard to get wrong (P0).** When the contestant knows two things differ (vertices/edges, positions/cuts,
absent/valid, per-test/shared, raw/compressed), the language keeps them apart. Arithmetic and mutation have
predictable semantics. Debug and release agree for valid programs. Hidden-complexity traps warn. P0 bug
classes: off-by-one, wrong index space, overflow, boundaries, multitest leakage, I/O and interactive
flushing, sentinels, complexity blowups, mutation surprises, modulus, stack depth, floating point.
*Test:* replay real bugs by class (target >= 90% handled), recording prevented / caught on failing run /
left to diagnose. [proposed] Escape hatches (`@unchecked`, raw casts) in <= 5% of ports, else safety is
too strict.

**III. Performance and control (P0).** You can tell what allocates, copies, traverses or uses stack. You choose
algorithm, representation and width. At the same algorithm and layout, abstractions cost what direct C++
costs (domains, optionals, iterators, operation values: zero). Fewest bytes the types allow. No implicit
choice unless it majorizes (never worse on any axis), and it is shown. TLE/MLE/overflow estimates are
warnings. Never trapped: smooth path to raw arrays, bit tricks, inline C++.
*Test:* matched C++ comparison, runtime p50 <= 1.1x and max <= 1.5x [you], and every port reaches <= 1.1x
with local tuning in the language, never by rewriting in C++ [you]. [proposed] Peak memory p50 <= 1.0x, max
<= 1.2x; byte-size checks pass (`?Id` is 4 bytes, ranged `?Int` uses a niche); remaining checks cost <= 5% in
hot loops before `@unchecked`; emitted code inspected when surprising; `cp explain` shows bytes per element.

**IV. Doubt to evidence (P0).** Cheap to ask "is this observation true?", "where do two implementations first
disagree?", "which state first went wrong?". Instant-to-understand errors (one per root cause, naming the rule
and the fix); traps with local values in source terms; low-noise HTML trace; built-in brute swap (`@brute` in
the same file), generators, checkers, shrinking stress, interactor harness; exploration of small cases;
incomplete programs run; live editor feedback; deterministic; front end < 500 ms.
*Test:* time to diagnose a planted bug; time to counterexample a false conjecture. [proposed] Every trap shows
source line and values in source terms; stress finds a seeded bug in median <= 10 s and shrinks it; edit to
run <= 500 ms; a 1e6-step trace opens in <= 2 s.

**V. Dependable submission (P0).** Named judge profiles (CF, AtCoder, DMOJ, CSES, USACO, IOI graders) fix
compiler, flags, stack, widths, libraries, limits. One deterministic C++17 file, 64 KB hard limit (warn at
48 KB). No UB in emitted C++. Local runs mimic the profile. Emitted C++ compiles fast. Documented recursion
contract.
*Test:* run the actual bundled artifact per profile for ordinary, multitest, interactive and grader tasks;
GCC and Clang `-std=c++17`. [proposed] 100% of ports compile warning-free on both; UBSan/ASan clean on stress
inputs; debug and release outputs identical on all stress inputs; p95 emitted size <= 32 KB; judge compile time
<= 2x the original; byte-deterministic output.

**VI. Accumulate (P1).** Personal library abstractions get the ergonomics and performance of the stdlib;
old solutions rebuild byte-for-byte after breaking library changes (pins, `cp fix` codemods); libraries are
shareable. *Test:* change a substantial user library incompatibly and rebuild a pinned old solution. [proposed] `cp fix` migrates 100% of corpus solutions; a
pinned build is byte-identical a year later.

**C. Coherent (constraint on every goal, never traded in a ranking).** Every new concept says what existing
concepts fail to express, composes predictably, and justifies the rule it adds. Few general concepts;
general-purpose core with contest shapes in the stdlib; no stdlib privilege; one spelling and one vocabulary
per idea; canonical formatting; no special cases. A feature is not done until it works in every tool
(errors, interpreter and trace, stress, fmt, lint, LSP, explain, bundler, docs, profiler). Prefer removing
or unifying to adding. Conveniences are instances of general mechanisms: every file-level setting is a
scoped compile-time context a library declares (`with io(base: 1)`), overridable per call, open to user
libraries. Concept count is a review aid, not a target.
*Test:* [proposed] every feature passes the tool-coverage checklist; one spelling per idea, lint-enforced
across ports; a new concept must remove friction in >= 5 ports; concept count reported every round.

## Decision rules

1. Gate: correct specified behavior and a viable submission on the target profile.
2. Then compare contestant time, error rate and resource use; search for a design that wins on all before
   trading.
3. Safety cost = frequency x distance from relevant code x concepts to remember (not tokens).
4. Use the clearer algorithm when it fits with comfortable headroom; the language must make the faster one
   equally direct.
5. A correctness feature that costs hot-loop time needs a zero-cost form.
6. Explicit beats implicit unless the implicit choice majorizes. Name the algorithm; never pick one secretly.
7. Don't add core syntax for what one stdlib facility covers.

## Measurement

End-to-end benchmark above all: unseen tasks (trivial, boundary-heavy, performance-tight, algorithm change);
fraction accepted in fixed time, time to AC, where time went. Size is three metrics: contestant-written code
(primary), concepts needed, emitted C++ bytes. Code moved into the stdlib counts as a win, weighted well below
an equal cut in user code. Goals, commitments and hypotheses stay separate: agents report evidence against a
commitment, never silently override it.

## Locked calls (Ryan)

- Unproven checks (`a[i]`) stay in the submission and trap; greppable `@unchecked` scope removes them.
- Overflow wraps (defined) in release, traps in debug; ranges prove most arithmetic safe.
- Domains are scoped to the block that creates them; per-test indices cannot leak into static data.
- Pointer structures: 4-byte typed handles via stdlib `Arena<T>`; no pool syntax.
- Generics checked at definition, with excellent errors; libraries can attach custom diagnostics.
- `==` on floats rejected by default (lint-configurable per file or site); `approx` or exact types.
- 1-based input converted at the boundary; viewer can display 1-based. Spelled as a scoped default
  (`with io(base: 1)`), not a one-off attribute. [proposed] `with` sets lexical context (reaching
  library internals only through declared `uses`); `@` only marks what a declaration is.
- Bools print via `yes(b)` (profile spelling, local override); bare `print(Bool)` is an error.
- `@brute` lives in the same file, stripped from submission, used by stress.
- Indentation syntax, `=>` arms, `and/or/not`, floor division, interpolated strings; `Int` is i64 with a
  reserved niche so `?Int` is free; file is a function body, `@multitest`, `static let`, top-level `return`.
- Ordered maps by default, hash maps opt-in; dense structures explicit.
- Options as named arguments (`runs(a, by: f)`), not `_by` variants; `runs` yields ranges.
- No `pure` keyword; `@memo` rejects functions that do I/O.
- Math-notation fidelity, checked claims, adversarial robustness: lower priority.
