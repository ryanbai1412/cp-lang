# AGENTS.md

Read this first if you are working on CP Lang, agent or person: what we are
building and the rules every change follows. How we work (settled decisions
and the loop) is in `plan/OPERATING.md`; Ryan's own words are in
`plan/agent-context.md`; every design call and its reason is in
`decisions.md`; the language is in `plan/1x-*.md`.

## The mission

Minimize the time from an algorithmic idea to an accepted submission the
contestant understands and trusts. Quick to write and to revise, common
mistakes hard to write, failures easy to explain, with the speed and control
of hand-written C++ and a predictable path onto the judge. Judge the whole
contest attempt (drafting, changing your mind, debugging), not only the
finished program.

The feel we are after, in Ryan's words: "when i use the language it should
just feel like the pieces fit together beautifully and everything just works
... i never should need to feel like i have to hack stuff together."

## The goals

Each goal has a test; a claim without a passing test is not a claim.

1. **Express and revise.** Trivial problems take about three lines. Hard
   solutions read like the editorial. A changed requirement (the DP gains a
   dimension, an array becomes a map) is a local edit. Every convenience has a
   one-step local override, so the cold path (mixed 0- and 1-based input, odd
   output formats) costs one edit, never a rewrite. Test: contestant-written
   tokens median at most 0.6x the C++ original, none above 1.0x.
2. **Hard to get wrong.** When the contestant knows two things differ
   (vertices and edges, positions and cuts, absent and valid, per-test and
   shared), the language keeps them apart. P0 bug classes: off-by-one, wrong
   index space, overflow, boundaries, multitest leakage, I/O and interactive
   flushing, sentinels, complexity blowups, mutation surprises, modulus,
   floating point. Index safety is one P0 among many.
3. **Performance and control.** Abstractions cost what direct C++ costs.
   You can see what allocates, copies or traverses. Fewest bytes the types
   allow. Test: runtime p50 at most 1.1x matched C++, max at most 1.5x, and
   every port can reach 1.1x by tuning in the language.
4. **Doubt to evidence.** Errors name the place, the rule and the fix, one per
   root cause. Traps show values in source terms. Brute, generator, checker,
   shrinking stress, interactor harness, HTML traces, debugger: all cheap.
   Front end under 500 ms.
5. **Dependable submission.** One deterministic C++17 file under 64 KB (warn
   at 48 KB), no undefined behaviour, judge profiles fix compiler, flags and
   stack size. The interpreter and the C++ agree on every program, including
   on input they reject.
6. **Accumulate.** Personal libraries get stdlib ergonomics and speed; old
   solutions keep building.

**Coherence is a constraint on every goal, never traded away.** Every new
concept says what existing ones fail to express and composes predictably.
Few general concepts, general-purpose core, contest shapes in the stdlib, no
stdlib privilege, one spelling per idea, no special cases. A feature is not
done until it works in every tool: errors, interpreter and trace, stress, fmt,
lint, LSP, explain, bundler, docs.

## Principles for every change

- **Rethink, don't translate.** A port asks how the idea is best expressed in
  this language, never how the C++ maps line by line.
- **Fix frictions generally.** Hold an extremely high bar on friction, and fix
  each one with a general mechanism that keeps the whole coherent. A helper
  with one use means the general mechanism is missing. Prefer removing or
  unifying to adding.
- **Small core, no privileged stdlib.** If the stdlib needs a power, user
  libraries get it too, or it becomes an explicit core feature.
- **Coherent stdlib.** One naming scheme, one way per job, guessable from its
  neighbours. Options are named arguments (`by:`), not `_by` variants.
- **Explicit unless it majorizes.** An implicit choice is allowed only when it
  is never worse on any axis, and `cpl explain` shows it. Name algorithms.
- **Honest measurement.** Code moved into the stdlib counts as a win, but far
  less than the same cut in user code. Performance compares matched
  algorithms. Claims cite a run.
- **Main stays green.** Every change lands with tests and passes
  `scripts/check.sh --fast`. Never weaken a test to land a change.
- **The spec is a starting point, not a cage.** Pick the better design when
  you find one, and log why in `decisions.md`.
- **Own the design.** Ryan delegated ordinary design decisions. Decide, log the
  reason and how to reverse it, and keep moving. Bring him only calls that
  change the language's character, one at a time, with a recommendation.
- **Nothing sensitive in the repo.** The history will be public: no internal
  hostnames, package mirrors, session links, secret names or credentials.
