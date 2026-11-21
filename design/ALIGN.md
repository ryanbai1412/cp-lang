# Alignment doc: goals, principles, trade-offs, co-design, components

How to use this: every item is a numbered statement. Items tagged **[you]** are things you already said
(quoted or paraphrased from our history); I treat them as locked unless you reopen them. Items tagged
**[default]** are my proposal. Reply in bulk, e.g. "3.4 no", "4.2 flip it", "6.7 yes but ...". Silence
counts as agreement. Once an item is agreed it becomes part of the constitution every design agent must
follow and cannot override.

The layers constrain downward: goals justify principles, principles decide trade-offs, trade-offs and
co-design constraints shape components, and individual decisions cite the items above them.

## 1. Mission and goals

Tags: **[you]** = your call; **[fb]** = from the feedback you forwarded (21:29); **[default]** = my proposal.

**Mission.** Minimize the time it takes a contestant to turn an algorithmic idea into an accepted submission
they understand and trust. Make the idea quick to express and revise, common mistakes hard to write, and
failures easy to explain. Give the contestant the performance and control of hand-written C++ and a
predictable path onto the judge. [fb]

**Judge the whole contest attempt, not the finished port.** [fb] A language can produce a beautiful final
solution and still cost ten minutes when the DP needs another dimension, the binary search must return an end
position, or the first idea was wrong. Revision and discovery are part of the mission, not extras.

**Structure.** Five contest goals (I-V, P0), one long-term goal (VI, P1), and one design constraint (C, P0 on
all of them). Each goal answers a different question; a design can pass any one and fail the rest. Old
`plan/00-vision.md` goals are noted as (G1)...(G14) for traceability.

- I Express and revise: does the code say the idea, and does a change of mind stay local?
- II Hard to get wrong: can the bug be written?
- III Performance and control: is it as fast and small as C++, and can you see and choose why?
- IV Doubt to evidence: when you're unsure, how fast do you get an answer?
- V Dependable submission: does what passed locally pass on the judge?
- VI Accumulate: do your techniques and old code keep paying off?
- C Coherent: a constraint on every design, not a goal ranked against them.

### I. Express and revise (P0)

What the language owes you: a trivial solution takes trivial code; a hard solution exposes its invariant,
recurrence or algorithm; you can start with a brute force, add structure when it earns its keep, and change
your mind locally. (G4, G12)

I.1 **[you] It just works.** The pieces fit together beautifully; you never feel you are hacking something
together.

I.2 **[you] Trivial problems are trivial.** A Div2 A is ~3 lines with no ceremony.

I.3 **[fb] Progressive adoption.** Extra structure pays for itself at the point you introduce it. A one-array scan
is effortless; in a graph problem, declaring `Vertex` and `Edge` once protects every relevant array, function
and loop. Target: one declaration per semantic index space, and two arrays over the same space work together
immediately. Domains are welcome when cheap to add [you].

I.4 **[default] Easy to change your mind.** Each fact lives in one place, so a DP gaining a dimension, an array
becoming a map, or a search returning an end position instead of an element is a local edit.

I.5 **[default] Reads like the editorial.** The code says what the solution is, not how C++ would encode it.

I.6 **[default] The statement maps straight to code.** Constraints and I/O formats are written once, as types,
and drive validation, storage width, generation and overflow proofs.

I.7 **[you] Guessable APIs.** Uniform naming and argument conventions; you can guess a function under stress.

I.8 **[you] Calm under pressure.** Low visual noise; readable and forgiving at 3am.

I.9 **[you] Natural recursion.** Local recursive functions and closures just work: no `std::function`, no
Y-combinator tricks.

I.10 **[you] Customizing an algorithm is cheap.** A segment tree with a custom op, or lazy propagation added to
it, is a few lines.

I.11 **[default] Imperative where the algorithm is imperative.** In-place DP tables and mutation are natural;
no forced functional style.

I.12 **[default] Output is as easy as input:** yes/no, joined lists, float precision.

I.13 **[you] Single-test and multitest feel identical** (also 3.7).

I.14 **[you, lower] Math notation fidelity.**

I.15 **[you] Graceful off the happy path.** Every convenience has a local override at the point of need, in one
obvious step; the cost of leaving the happy path is proportional to how far you leave it, never a cliff that
forces dropping the convenience file-wide. Example: mixed 0- and 1-based input under `@base(1)` uses
`read(base: 0)` on one line, a domain with written labels (`0..k`) keeps them, and `print(x, base: 0)`
overrides output. *Test:* every exemplar includes a cold-path variant, and it costs one local edit.

Test: [fb] the revision test. Give a contestant an unseen problem, then change one requirement; record where
they hesitate, consult docs, duplicate a fact, or rewrite unrelated code. Plus port reviews finding no
awkward spot left unfixed.

### II. Hard to get wrong (P0)

What the language owes you: when you know two things are different, the language helps keep them apart:
vertices and edges, positions and cuts, absence and a valid value, per-test and shared state. Arithmetic and
mutation have predictable semantics. (G3) [fb]

II.1 **[you] Correct by construction.** The P0 bug classes of layer 2 are hard to write, and where a bug can
exist the structure makes it obvious where.

II.2 **[you] Debug and release agree.** For valid programs the interpreter, debug build and submitted C++ give
the same results; the safety model (3.10) says exactly what each build checks.

II.3 **[you] Hidden-complexity traps warn:** `erase` from the front in a loop, repeated string concatenation,
whole-table clears per test.

II.4 **[you, lower] Checked claims.** Assumptions ("f is monotone", "answer fits in Int") are stated and checked
on random and small inputs.

II.5 **[you, lower] Adversarial robustness.** Hack-proof defaults (randomized hashing, safe sorting).

Test: [fb] replay real bugs by class (target: at least 90% handled), and record separately which were prevented
(can't be written), caught on a failing execution, or left for the contestant to diagnose.

### III. Performance and control (P0)

What the language owes you: you can tell what allocates, copies, traverses or uses stack; you choose the
algorithm, representation and numeric width the constraints demand; at the same algorithm and layout,
abstractions match direct C++. (G1, G2, G9) [fb]

III.1 **[you] As fast as hand-written C++.** No hidden copies, allocations, virtual calls or boxing; domains,
operation values, iterators and optionals cost nothing.

III.2 **[you] As small as hand-written C++.** Values take the fewest bytes their types allow (4 vs 8 bytes
matters): widths from ranges, niche optionals, flat grids, 4-byte handles.

III.3 **[you] You can see why.** Cost is predictable from reading the code; representations and lowering are
simple enough to picture (no exact C++ mapping required). `cp explain` shows representation and bytes per
element; a per-line profiler compares estimated and actual cost.

III.4 **[you] No implicit choices unless they majorize.** Representation follows the declared type; the compiler
may choose only when the choice is never worse on any axis, and shows it.

III.5 **[you] Estimates as warnings:** TLE, MLE and overflow risk.

III.6 **[default] Explicit control** of layout (SoA vs AoS, bitsets) and numeric width.

III.7 **[default] Fast I/O by default; large arrays never land on the stack.**

III.8 **[default] Never trapped.** A smooth path down to the metal: bit tricks, raw arrays, pointer-heavy
structures, inline C++, including existing C++ library headers [you].

Test: compare against matched C++ on hot paths and full solutions: within 1.1x time and at most 1.0x peak memory
and stack; inspect emitted code wherever a result surprises.

### IV. Doubt to evidence (P0)

What the language owes you: it is cheap to ask "is this observation true?", "where do these two
implementations first disagree?" and "which state first went wrong?". Exploration, brute swaps, shrinking,
traces and diagnostics all serve this. (G5, G10) [fb]

IV.1 **[you] Great errors.** Understood instantly; one root cause gives one error; each names the rule
involved and the fix.

IV.2 **[you] Great debugging.** Debug runs trap at the bug and show local values in source terms; an HTML trace
walks you through execution, filterable and low-noise.

IV.3 **[you] Checking is built in.** Brute swaps, generators, checkers, stress with shrinking, part-by-part
testing, a local interactor harness.

IV.4 **[default] Exploration is first-class.** Brute-force small cases, print tables, diff against a guess,
search for a counterexample to a conjecture.

IV.5 **[default] Incomplete programs still run.** Errors in unreached code become runtime traps.

IV.6 **[you] Live in the editor.** Sample results, types, ranges and widths inline as you type.

IV.7 **[default] Deterministic and fast.** Same input, same output and trace; seeded randomness; front end
under 500 ms; debug runs don't wait for g++.

Test: [fb] plant a plausible bug and measure time to a correct diagnosis; give a false conjecture and measure
time to a counterexample. Both on a named diagnostic benchmark, plus "would I know what to type next?" on a
corpus of error messages.

### V. Dependable submission (P0)

What the language owes you: a locally validated solution has a clear route to the judge; target compiler,
allowed dependencies, entry point, I/O protocol and source budget are known; local execution faithfully
predicts judged behavior for valid programs. (G6) [fb]

V.1 **[fb] Named judge profiles,** not "any judge": Codeforces, AtCoder, DMOJ, CSES, USACO, and IOI-style
graders to start. Each fixes compiler and flags, stack limit, `long` width, available libraries (ACL, Boost,
`__int128`) and source limit.

V.2 **[you] One deterministic C++17 file** under the profile's limit (64 KB hard, warning at 48 KB). Over the limit
is a clear bundling error, never a silent truncation.

V.3 **[you] Run locally as the judge would:** same flags, stack limit and integer widths per profile.

V.4 **[you] No UB in generated C++.** Signed overflow, bad shifts and invalid comparators never reach `-O2`.

V.5 **[you] Generated C++ compiles fast** within judge compile limits.

V.6 **[fb] A documented recursion contract per profile** (supported depth and how deeper recursion is handled),
replacing "deep recursion never breaks".

Test: build and run the actual bundled artifact under each profile, for ordinary, multitest, interactive and
grader tasks; compare local and compiled behavior; every port compiles on GCC and Clang at `-std=c++17`.

### VI. Accumulate (P1)

What the language owes you: a technique you use repeatedly becomes a personal library abstraction with the
ergonomics and performance of the stdlib; you can improve that library and still reproduce an old submission.
(G11) [fb]

VI.1 **[you] Your library composes like the stdlib,** across 10-line and 500-line programs.

VI.2 **[you] Old solutions keep building** after breaking library changes: pinned versions, reproducible builds
(same source and lock, same bytes), and `cp fix` codemods to migrate.

VI.3 **[you] Share and import others' libraries** (e.g. ecnerwala's).

VI.4 **[you, lower] Language editions.** Version stamps now; a compatibility promise once stable.

Test: [fb] implement a substantial abstraction outside the compiler, use it across problems, change it
incompatibly, and rebuild a pinned old solution byte-for-byte.

### C. Coherent (constraint on all goals, P0)

C is not in any ranking [fb, you]: "coherence beats correctness" would let an agent keep a tidy rule that keeps
causing wrong answers. Its job is concrete instead. (G7, G8)

C.1 **[fb] Every new concept earns its place:** it says what existing concepts fail to express, composes
predictably with the relevant features, and justifies the extra rule a contestant must remember. A single
overloaded concept can be harder to learn than two precise ones.

C.2 **[you] Few concepts, each general;** not a pile of stuff thrown together.

C.3 **[you] General-purpose core, contest shapes in the stdlib.** ~90% of the stdlib is ordinary code on a small
listed intrinsics layer (also 3.5).

C.4 **[you] No stdlib privilege.** User libraries get every power the stdlib has: operators, domains, trace
rendering, `cp explain`.

C.5 **[you] One design across the whole toolchain.** A feature isn't done until it works in every tool: errors,
debug interpreter and HTML trace, stress and brute swap, formatter, linter, LSP and highlighting, `cp explain`,
bundler, docs generator, profiler and coverage. A feature a tool can't support well is redesigned, not
special-cased in that tool.

C.6 **[default] Same idea, same spelling; one way to do each thing; one vocabulary** across spec, errors, trace
and docs; one canonical format with no formatter options [you].

C.7 **[default] No special cases, so no surprises.** Familiar meaning where it matters: floor division, no
implicit narrowing, no truthiness. Exceptions are logged as design debt.

C.8 **[default] Prefer removing or unifying to adding.**

C.9 **[you] Conveniences are instances of general mechanisms.** No one-off attribute: every file-level
setting (`base`, yes/no spelling, precision, separators, lint levels, checks) is a compile-time context a
library declares, set lexically with `with lib(arg: v)` on a file or block and overridden by passing the
argument at one call; it reaches library internals only through a declared `uses`. `@` only marks what a
declaration is. User libraries declare their own (`with mylib(eps: 1e-9)`). [proposed, exemplar round 6]

Test: [fb] the meaningful feature interactions are listed and each has a test (replacing "every feature works
with every other"); every feature has one coverage test per tool; whole-language reviews hunt overlaps and odd
ones out. Concept count is a review aid, not a target.

### How we measure (applies to all goals)

**[fb] One end-to-end benchmark above all the goal tests.** Unseen tasks, including a trivial implementation, a
boundary-heavy task, a performance-tight task and a task that forces an algorithm change. Record the fraction
accepted within a fixed time, time to AC, and where the time went. Agents serve as proxy contestants; you
spot-check. Finished ports from your and ecnerwala's submissions give coverage and C++ baselines but can't
show time saved during discovery.

**[fb] Three size metrics, kept separate:** code the contestant writes, concepts they must know, and emitted
C++ bytes. [you] Code moved into the stdlib is a win, but a much smaller one than shorter contestant-written
code: if `runs(a)` replaces a fiddly loop the contestant would have debugged, it counts, weighted well below
an equal cut in user code, and its cost still shows in concepts and emitted bytes.

**[fb] Goals, commitments and hypotheses stay at different levels.** "Prevent accidental overflow" is a goal.
"`Int` is i64 with one reserved value" is a commitment (layer 6 and the decision log, each linked to the goals
it serves). "Range declarations will usually prove products safe without annotations" is a hypothesis to test
on ports. Agents report evidence against a commitment; they never silently override it.

**[you] Evidence or it isn't true.** Zero-cost by benchmarks and emitted-code inspection; regressions tracked
across rounds; validation mostly ensures performance.

### Scope and non-goals

Consequences, not goals: learnable in an afternoon and teachable (follows from C and I); implementable and
thoroughly testable by us (follows from C).

In scope, lower priority: team and onsite contests (readable C++, printable notebook), rules-compliant output,
problem setting (validators, generators, checkers, interactors), precomputation within 64 KB, interviews,
Advent of Code and Project Euler, a searchable personal library, learning from your bugs, code good enough
for editorials.

Non-goals: production or systems programming; I/O beyond stdin/stdout and interactors; running submissions on
anything but compiled C++ (the interpreter is local only); dynamic typing, reflection, inheritance; copying
C++ semantics where C++ is a trap (division, overflow, uninitialized memory); fewest keystrokes at the cost
of clarity.

## 2. P0 bug classes the language must make hard

Each is a concrete promise; a design passes only if it removes or exposes the class.

2.1 **[you] Off-by-one** in ranges, loops, windows, cuts and binary search.

2.2 **[you] Wrong index space:** indexing array `a` with an index meant for `b`, row/column swaps, 0- vs
1-based mixups, compressed vs raw coordinates, vertices vs edges, positions vs cuts.

2.3 **[default] Overflow:** silent wraparound of intermediate products and sums.

2.4 **[default] Boundaries and edge cases:** empty inputs, n = 1, neighbors off the grid, wraparound on cycles.

2.5 **[default] Multitest state leakage:** globals not reset, arrays sized for the previous test, O(maxN)
clearing per test.

2.6 **[default] I/O mistakes:** wrong output format, forgotten flush in interactive problems, reading the wrong
count, 1-based input mistaken for 0-based.

2.7 **[default] Uninitialized and sentinel values:** `-1` meaning "none", `INF` that overflows when added.

2.8 **[default] Accidental complexity blowups:** O(n) copies of big values, clearing whole tables, map where an
array was meant, hidden quadratic behavior.

2.9 **[default] Mutation surprises:** aliasing, iterator invalidation, modifying a container while iterating.

2.10 **[default] Wrong modulus or missing mod reduction** in modular arithmetic.

2.11 **[default] Stack overflow** from deep recursion (DFS at n = 1e6).

2.12 **[you] Floating point:** epsilon comparisons, precision loss, choosing exact integer/rational geometry.

## 3. Principles (rules that decide design questions)

3.1 **[you] Nothing secret under the hood.** Representation follows the declared type; the compiler never
silently swaps a set for a bitset or a map for an array. Dense forms are explicit types.

3.2 **[default] Name the algorithm.** `bfs`, `bisect`, `SegTree` are explicit; the compiler specializes them but
never picks a different algorithm for you.

3.3 **[default] The obvious way is the correct and fast way.** If the natural code is subtly wrong or slow, that
is a design bug.

3.4 (moved to C.6)

3.5 **[you] Small general-purpose core; contest shapes in the stdlib.** Don't bake CP-specific features into the
language when a library can do it; users should be able to reimplement ~90% of the stdlib at full speed.

3.6 **[you] Explicit, greppable escape hatches** for checks (domain checks, bounds, overflow).

3.7 **[you] Single-test and multitest feel identical.** The file is a function body; decorators change how it
is driven, never what it means.

3.8 **[default] Types carry the proofs.** Ranges on input give storage width, overflow proofs and validation;
domains give index safety. Proofs come from declarations you would write anyway, not annotations.

3.9 **[default] Costs are visible.** Anything O(n) or allocating looks like it (`.copy()`, named algorithms);
anything that looks O(1) is O(1).

3.10 **[fb] Every checked operation answers three questions in the spec:** what does it mean, what can the
compiler prove, and which checks does each build run? For `a[i]` and `x * y` the spec answers all three.
"Debug checks it" alone leaves you guessing what the submitted program does. Prefer compile-time errors, then
traps, never silent wrongness. This is the most consequential open issue (Q6).

3.11 **[you] Warnings, not errors, for estimates** (TLE, MLE, overflow risk).

3.12 (replaced by C.1 and the listed-interactions test in C)

3.13 (moved to C.8)

## 4. Trade-off orderings (what wins when goals collide)

Decision rules replace a global ranking of goals [fb]:

4.0 **[fb] Gate, then compare.** First require correct specified behavior and a viable submission on the target
profile. Then compare designs on contestant time, error rate and resource use, searching for one that wins on
all three before trading.

4.1 **[fb] Safety cost is measured by frequency, distance from the relevant code, and concepts the reader must
remember,** not tokens. (Replaces the old one-extra-token rule.)

4.2 **[fb] Use the clearer algorithm when it fits with comfortable headroom;** when performance matters, the
language owes an equally direct way to write the faster one.

4.2a **[old G-doc] Zero-cost exception.** A correctness feature that costs measurable time in hot code must have
a zero-cost form: proven at compile time, or checked only where 3.10 says so.

4.3 **[default] Explicit beats magic** when the magic would change cost or representation; terse inference is
fine when it changes neither (local type inference, typed `read()`).

4.4 **[default] Core simplicity beats stdlib convenience**: never add core syntax for something one stdlib
function covers.

4.5 **[default] Familiarity (Python/C++/Rust) beats novelty** unless the novel form removes a P0 bug class.

4.6 **[default] Generated C++ size is a hard constraint, generated C++ readability a soft one.**

4.7 **[default] Compile speed beats exotic static analysis.** Analyses must be linear-ish.

## 5. What we co-design with (constraints we must fit)

5.1 **[you] Judges:** C++17 output, 64 KB source, judge compilers (g++ -O2), stack limits, interactive
protocols, IOI-style graders.

5.2 **[you] Your workflow:** a local library on disk with breaking changes, per-problem `i1..i5` inputs,
`cp run`, stress tests, brute-force swaps, part-by-part testing.

5.3 **[you] Tooling:** debug interpreter, HTML trace, stress with shrinking, formatter, linter, LSP/VS Code.
Language features must give tools what they need (for example, the trace must capture domains and
names).

5.4 **[default] The compiler:** every feature must be type-checkable locally, lower to C++ predictably, and be
explainable in an error message.

5.5 **[default] LLM writability:** models should write correct code in it from the spec alone (this is also how
we validate the spec).

5.6 **[default] The corpus:** your submissions and ecnerwala's are the ground truth for what problems need.

## 6. Components and their hard requirements

6.1 **Syntax.** [you] indentation, no semicolons, `=>` arms, `and/or/not`, interpolating strings, `[V] v => e`
tables, floor division, `@get`. [default] no feature needs more than one new keyword.

6.2 **Types.** [you] `Int` is i64 with `Int.MIN = -(2^63-1)` so `?Int` is free; anonymous records; ADTs and
optionals instead of sentinels. [default] range types choose storage width; no implicit numeric conversions
except widening.

6.3 **Index domains.** [default] every array index has a domain type; mixing domains is a type error; shapes
(grid, ring, implicit tree, cuts) remove neighbor and wraparound arithmetic; compressed coordinates are a
fresh domain.

6.4 **Sequences and views.** [you] `runs` returns ranges, options via named args (`runs(a, by:)`) instead of
`_by` variants. [default] views are zero-cost and never allocate unless named (`.collect()`/`.sorted()`).

6.5 **State and ownership.** [you] variables own, params borrow, `mut` at declaration and call site, explicit
`.copy()`, defer and scoped undo/rollback, typed pools. [default] no lifetimes in user code.

6.6 **Abstraction.** [you] a monoid is an operation over a type (operation values), laws come naturally from
types. [default] traits only for one canonical instance; generics monomorphize.

6.7 **Program modes.** [you] file as function body, `@multitest`, `static let`, top-level `return`, library
files; interactors and checkers co-located; `#ifdef LOCAL` equivalent. [default] `@grader` treats the file as
`init`'s body.

6.8 **I/O.** [you] clean typed reading is nice but P2. [default] output format is explicit and checked by tools.

6.9 **Stdlib.** [you] good basics, not fancy; binary search, neighbors, prefix sums, modint are stdlib, not core;
`inf` and nice builtins. [default] every function documents cost; reusable shapes, not problem patterns.

6.10 **Diagnostics.** [you] crystal clear. [default] every error names the domain, range or rule involved and
suggests the fix.

6.11 **Safety model.** [you] bound checks must not crush perf. [fb] specified per operation by the three
questions of 3.10; open (Q6).

6.12 **Codegen.** [you] zero-cost, under 64 KB, version-stamped output. [default] only used stdlib code is
emitted.

6.13 **Tooling and debugging.** [you] as in 5.2-5.3; `--trace` filterable and low-noise.

6.14 **Docs and learning.** [you] beautiful docs site and a playground with CP problems.

## 7. Decisions

Each decision records: the options, the choice, the goal/principle items it cites, the problems that
motivated it, and status (proposed / aligned / locked). These live in the spec's design log; anything
contradicting a locked item above is rejected automatically.

## 8. Taste exemplars

Next step after you've gone through this: about 10 problems, each with 2-3 solution variants that differ
in exactly one design choice (e.g. bisect vs two pointers, cut domains vs plain ints, `match` vs `??`).
[fb] For at least half, show the first draft, the surprise, and the edit: e.g. a binary search guaranteed to
find an element, then the problem changes so every element may be too small. How the code absorbs the change
says more about domains, cuts and optionals than three polished spellings. Your picks become locked
exemplars that agents imitate.

## Questions where your opinion matters most

Q1 (4.1) How much ceremony will you pay for safety? E.g. `for i in V` with `a[i]` typed by domain vs plain
`for i in 0..n`. Is a domain declaration per array family acceptable?

[fb] vote: yes to one domain declaration per meaningful index space; ordinary loops and indexing infer it
wherever possible; drop the one-extra-token rule.

Q2 (4.2) When the clear version is O(n log n) and the fast one O(n), which should the ideal solution use?

[fb] vote: the clearest solution with comfortable headroom under the actual constraints; two pointers if
equally clear, bisect if it makes the proof much easier and fits.

Q3 (C.1 vs I.2) Is it OK for the stdlib to have many small named helpers (`longest_window`, `runs`,
`bisect_index`) if each removes a bug class, or should it stay minimal?

[fb] vote: small composable helpers with precise contracts. `runs` earns its place; `longest_window` must
show its validity assumptions and endpoint semantics are easy to infer before admission.

Q4 (6.3) Should domain arithmetic (`x + 1`) be allowed and checked, or must you go through `x.next`?

[fb] vote: allow arithmetic with a clear meaning (distance between positions in one ordered domain).
For advancing, distinguish "next element if it exists" from "the cut just past this element"; work
through the last element and the empty array before locking `+ 1` or `.next`.

Q5 (2.3) Should overflow checks be on in debug by default (cost: slower debug runs)?

[fb] vote: yes, on by default; report the operation, operands and type; intentional wrapping and modular
arithmetic get explicit semantics of their own.

Q6 (3.10) For `a[i]`, `x * y` and the other checked operations: what does each build check, and what
does the submitted program do when an unproven check would fail?

## Hard questions (answered 2026-11-20 21:36)

H1 **[you] Unproven checks stay in the submission and trap** (`a[i]` the compiler can't prove in range). A
greppable `@unchecked` scope removes them in hot loops. Debug and release never disagree. Resolves Q6 for
indexing.

H2 **[you] Overflow: wrap (defined, no UB) in release, trap in debug.** Ranges prove most arithmetic safe; a
program that passes debug behaves identically in release. Resolves Q5 and Q6 for arithmetic.

H3 **[you] Domains are scoped to the block that creates them.** A per-test `Vertex` cannot leak into static
data; static data uses plain ranges.

H4 **[you] Pointer-heavy structures use 4-byte handles, with no special pool syntax for now** (21:40). They are
an ordinary stdlib growable array of structs with a typed index: `let nodes = Arena<Node>()`, `nodes.push(...)`
returns a 4-byte `Id` unique to that arena, `nodes[x].left`. Scoped like any value. A pool/pool-kind design
with field syntax and hooks was considered and deferred as too complex.

H5 **[you] Generics are checked at definition with excellent errors;** local helper bounds may be inferred.
Libraries (stdlib and yours equally) can attach custom diagnostics to their bounds, like Rust's
`#[diagnostic::on_unimplemented]`.

H6 **[you] `==` on floats is rejected by default;** configurable by a lint setting per file or at the use site.
Use `approx(a, b, eps)` or exact types.

H7 **[you] Convert 1-based input at the boundary;** everything inside is 0-based. The debug viewer and trace have a
setting to display indices 1-based.

H8 **[you] Brute-force solutions live in the same file** (`@brute`), are stripped from the submission, and are
used automatically by stress testing.
