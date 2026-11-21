# AGENTS.md

Read this first if you are working on CP Lang, agent or person: what we are
building, what it owes the contestant, and the rules every change follows.

Then read, in order:
1. `plan/OPERATING.md`: settled decisions and the loop we work in.
2. `plan/DOMAINS.md`: each domain's north star, status and next steps;
   find yours.
3. `design/README.md`: the target language (spec, goals with tests, core
   mechanisms, exemplars) and `design/DELTA.md`, the gap from today.
4. `decisions.md`: every design call, its reason and how to reverse it.
5. `REPO.md`: where everything lives.
6. `plan/PORTING.md`: if you port solutions, the bar and the review.

`plan/agent-context.md` has Ryan's own words; `plan/1x-*.md` describes the
language as implemented today.

## The mission

Minimize the time from an algorithmic idea to an accepted submission the
contestant understands and trusts. Quick to write and to revise, common
mistakes hard to write, failures easy to explain, with the speed and control
of hand-written C++ and a predictable path onto the judge.

Judge the whole contest attempt, not the finished program. A language can
produce a beautiful final solution and still cost ten minutes when the DP
needs another dimension, the binary search must return an end position, or
the first idea was wrong. Revision and discovery are part of the mission.

The feel we are after, in Ryan's words: "when i use the language it should
just feel like the pieces fit together beautifully and everything just works
... i never should need to feel like i have to hack stuff together."

## The goals

Five contest goals, one long-term goal, and one constraint on all of them.
Each answers a different question, and a design can pass any one and fail the
rest. Each has a test; a claim without a passing test is not a claim.

1. **Express and revise.** Does the code say the idea, and does a change of
   mind stay local?
   - Trivial problems take about three lines. Hard solutions expose their
     invariant, recurrence or algorithm and read like the editorial.
   - Each fact lives in one place: the DP gains a dimension, an array becomes
     a map, a search returns an end position, and it is a local edit.
   - Structure is adopted progressively and pays for itself where it is
     introduced: one declaration per index space protects every array over it.
   - Constraints and formats from the statement are written once, as types,
     and drive validation, storage width, generation and overflow proofs.
   - Guessable APIs, natural recursion and closures, cheap customization of an
     algorithm, imperative code where the algorithm is imperative, output as
     easy as input, single-test and multitest identical. Calm at 3am.
   - Graceful off the happy path: every convenience has a one-step local
     override, so the cold path (mixed 0- and 1-based input, odd output
     formats) costs one edit, never a file-wide rewrite.
   - Test: contestant-written tokens median at most 0.6x the C++ original,
     none above 1.0x; a requirement change touches one chunk; the revision
     test (unseen problem, then one requirement changes) finds no hesitation,
     duplicated fact or unrelated rewrite.
2. **Hard to get wrong.** Can the bug be written?
   - When the contestant knows two things differ (vertices and edges,
     positions and cuts, absent and valid, per-test and shared, raw and
     compressed), the language keeps them apart.
   - Arithmetic and mutation have predictable semantics. Debug and release
     agree for valid programs. Hidden-complexity traps warn.
   - P0 bug classes, each a concrete promise: off-by-one; wrong index space;
     overflow; boundaries and edge cases; multitest state leakage; I/O and
     interactive flushing; sentinels and uninitialized values; accidental
     complexity blowups; mutation surprises (aliasing, invalidation); wrong
     or missing modulus; floating point. Index safety is one P0 among many.
   - Test: replay real bugs by class, at least 90% handled, recording which
     were prevented, caught on a failing run, or left to diagnose.
3. **Performance and control.** Is it as fast and small as C++, and can you
   see and choose why?
   - At the same algorithm and layout, abstractions cost what direct C++
     costs: domains, optionals, iterators and operation values are free.
   - Values take the fewest bytes their types allow. You choose algorithm,
     representation and width; you can see what allocates, copies or
     traverses. Fast I/O by default; large arrays never on the stack.
   - Never trapped: a smooth path down to raw arrays, bit tricks and inline
     C++.
   - Test: runtime p50 at most 1.1x matched C++, max at most 1.5x, and every
     port can reach 1.1x by tuning in the language.
4. **Doubt to evidence.** When you are unsure, how fast do you get an answer?
   - Cheap to ask "is this observation true?", "where do two implementations
     first disagree?", "which state first went wrong?".
   - Errors are understood instantly: one per root cause, naming the place,
     the rule and the fix. Traps show local values in source terms.
   - Brute swaps, generators, checkers, shrinking stress, an interactor
     harness, small-case exploration, HTML traces and the debugger are all
     cheap. Incomplete programs still run. Everything is deterministic.
   - Front end under 500 ms; live feedback in the editor.
   - Test: time to diagnose a planted bug; time to a counterexample for a
     false conjecture.
5. **Dependable submission.** Does what passed locally pass on the judge?
   - Named judge profiles fix compiler, flags, stack size, widths, libraries
     and limits; local runs mimic the profile.
   - One deterministic C++17 file under 64 KB (warn at 48 KB), no undefined
     behaviour, fast to compile.
   - The interpreter and the C++ agree on every program, including on input
     they reject.
6. **Accumulate.** Do your techniques and old code keep paying off?
   - Personal libraries get the stdlib's ergonomics, speed and powers.
   - Old solutions rebuild byte for byte after breaking library changes,
     with pins and codemods to migrate.

**Coherence is a constraint on every goal, never traded away.**
- Every new concept says what existing ones fail to express, composes
  predictably, and justifies the rule a contestant must remember.
- Few general concepts. General-purpose core, contest shapes in the stdlib,
  no stdlib privilege. One spelling and one vocabulary per idea across spec,
  errors, trace and docs. No special cases; exceptions are logged as debt.
- Conveniences are instances of general mechanisms, never one-off
  attributes, and user libraries can declare their own.
- A feature is not done until it works in every tool: errors, interpreter
  and trace, stress, fmt, lint, LSP, explain, bundler, docs, profiler. A
  feature a tool can't support well is redesigned, not special-cased there.

Out of scope: production and systems programming, I/O beyond stdin, stdout
and interactors, running submissions on anything but the compiled C++,
dynamic typing, inheritance, copying C++ semantics where C++ is a trap, and
fewest keystrokes at the cost of clarity.

## Design rules

These decide design questions.

- **Rethink, don't translate.** Ask how the idea is best expressed in this
  language, never how the C++ maps line by line. The best program may look
  nothing like the original.
- **Fix frictions generally.** Hold an extremely high bar on friction, and fix
  each one with a general mechanism that keeps the whole coherent. A helper
  with one use means the general mechanism is missing. Prefer removing or
  unifying to adding.
- **The obvious way is the correct and fast way.** If the natural code is
  subtly wrong or slow, that is a design bug.
- **Small core, no privileged stdlib.** If the stdlib needs a power, user
  libraries get it too, or it becomes an explicit core feature. Never add
  core syntax for what one library facility covers.
- **Coherent stdlib.** One naming scheme, one way per job, guessable from its
  neighbours. Options are named arguments (`by:`), not `_by` variants. Every
  function documents its cost.
- **Nothing secret under the hood.** Representation follows the declared
  type. An implicit choice is allowed only when it is never worse on any axis
  (it majorizes), and `cpl explain` shows it. Name the algorithm; the
  compiler never picks a different one for you.
- **Costs are visible.** Anything O(n) or allocating looks like it; anything
  that looks O(1) is O(1).
- **Types carry the proofs.** Ranges and domains from declarations you would
  write anyway give widths, overflow proofs, validation and index safety.
  Prefer a compile-time error, then a trap, never silent wrongness. Escape
  hatches are explicit and greppable.
- **Every checked operation answers three questions:** what it means, what
  the compiler can prove, and what each build checks.
- **Gate, then compare.** First correct behaviour and a viable submission;
  then contestant time, error rate and resource use, searching for a design
  that wins on all three before trading. Safety cost is frequency times
  distance from the relevant code times concepts to remember, not tokens.
- **Zero-cost safety.** A correctness feature that costs hot-loop time needs a
  zero-cost form.
- **Familiar unless novelty removes a P0 bug class.** Familiar from Python,
  C++ and Rust wins otherwise.
- **Compile speed beats exotic analysis.** Analyses stay roughly linear.
- **Writable from the docs.** A model or a person should write correct code
  from the documentation alone; that is also how we test the documentation.

## How every agent works

- **Evidence or it isn't true.** Never report something as working, fast or
  fixed without a run that shows it. Claims cite the run. Performance
  compares matched algorithms. Code moved into the stdlib counts as a win,
  but far less than the same cut in user code.
- **Goals, commitments and hypotheses stay separate.** Report evidence
  against a commitment; never silently override it.
- **Read before you write.** Check how the codebase already does it, reuse
  what exists, and follow the surrounding style.
- **Fix the cause, not the symptom.** A failing port is a question about the
  language, the stdlib or a tool; find which before changing anything.
- **Every fix lands with its test.** The port or snippet that exposed a
  problem becomes a regression test. Never weaken a test to land a change.
- **Main stays green.** Small, focused commits, each passing
  `scripts/check.sh --fast`.
- **Leave it better.** If something you touch is confusing, fix it or log it;
  don't route around it.
- **Ask what the maximal version is.** For whatever you are building, ask
  what the most ambitious complete version looks like and what the next step
  toward it is.
- **Own ordinary decisions.** Decide, log the reason and how to reverse it in
  `decisions.md`, and keep moving. Bring Ryan only calls that change the
  language's character, one at a time, with a recommendation. The spec is a
  starting point, not a cage.
- **Report crisply.** Say what changed, the evidence, what is still open, and
  every assumption you made.
- **Nothing sensitive in the repo.** The history will be public: no internal
  hostnames, package mirrors, session links, secret names, credentials or
  personal paths.

## Commits

- Subject: `area: summary`, lowercase imperative, at most 72 characters,
  e.g. `stdlib: add interval map` or `port(cf2237h): rethink with interval
  map`. A body, when needed, says why, not what.
- Areas: `syntax`, `check` (types and analysis), `codegen`, `interp`,
  `stdlib`, `diag`, `lsp`, `editors`, `fmt`, `lint`, `stress`, `judge`,
  `tools`, `cli`, `docs`, `site`, `design`, `plan`, `port`, `frictions`,
  `bench`, `factory` (scripts, briefs, retros), `ci`, `deps`, `revert`. An
  optional scope in parentheses narrows it.
- One logical change per commit, each green on its own; a port and its
  friction report go together. Land on `main` with `scripts/land.sh`,
  which gates the exact rebased tip; never rewrite pushed history without
  Ryan's say-so.
- `scripts/install-hooks.sh` turns on the hooks: `pre-commit` runs the fast
  check (including the public-history scan: only hosts in
  `scripts/public-hosts.txt`, no home paths, nothing from the local
  `.git/info/sensitive-words`), and `commit-msg` checks the subject.
