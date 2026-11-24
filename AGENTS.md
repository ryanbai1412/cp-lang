# AGENTS.md

Read this first if you are working on CP Lang, agent or person: what we are
building, what it owes the contestant, and the rules every change follows.

The handbook (`handbook/README.md`) is the home for how we work. Read, in
order:

1. `handbook/product/`: `vision.md` (who CP Lang is for and why),
   `journeys.md` (the journeys we owe them) and `milestones.md` (the
   milestone every lane works toward). When choosing what to do next,
   unblock a step of the milestone's acceptance scenario first.
2. `handbook/principles.md` (summarised below) and
   `handbook/process/handbook-and-rfcs.md`: how the handbook is kept true,
   when to write an RFC, and the escape hatch.
3. `handbook/process/`: `factory.md` (the loop), `ownership.md` (who
   decides), `change.md` (how a change flows), `quality.md`.
4. `handbook/domains/README.md`: find your domain, read its charter, then
   `plan/domains/<domain>/` and `plan/QUESTIONS.md`.
5. `handbook/language/` and `spec/` (`spec/core/`, `spec/lib/`,
   `spec/tools/`, with `std/`): handbook leads design; spec/ is its
   downstream exhaustive form. The two change together; a disagreement is a
   bug to resolve, not a rule to pick from.
6. `plan/STATUS.md` (generated: every domain's metrics and open questions)
   and `spec/README.md`: where the work stands today.
7. As needed: `handbook/design/` (architecture; the target is
   `handbook/design/architecture.md`),
   `REPO.md` (where everything lives), `decisions.md` (every design call,
   searchable), `handbook/process/porting.md` (porting solutions).

`plan/agent-context.md` has Ryan's own words; historical design debates live
under `research/design-history/`.

## The mission

Make turning an algorithmic idea into a correct, efficient, accepted
solution as fast, natural, and enjoyable as possible. We build for
experienced competitive programmers (about Codeforces 1500+) who would make
CPL their primary language for contests, upsolving and training, in place
of a C++ setup tuned over years. Quick to write and to revise, common
mistakes hard to write, failures easy to explain, with the speed and control
of hand-written C++ and a predictable path onto the judge.

Judge the whole contest attempt, not the finished program. A language can
produce a beautiful final solution and still cost ten minutes when the DP
needs another dimension, the binary search must return an end position, or
the first idea was wrong. Revision and discovery are part of the mission.

The feel we are after, in Ryan's words: "when i use the language it should
just feel like the pieces fit together beautifully and everything just works
... i never should need to feel like i have to hack stuff together."

## Principles in brief

The goals, the design rules and how every agent works are in
`handbook/principles.md`; read it once in full. The rules in one line each:

- Rethink, don't translate.
- Fix frictions generally.
- The obvious way is the correct and fast way.
- Small core, no privileged stdlib.
- Coherent stdlib.
- Nothing secret under the hood.
- Costs are visible.
- Types carry the proofs.
- Every checked operation answers three questions.
- Gate, then compare.
- Zero-cost safety.
- Familiar unless novelty removes a P0 bug class.
- Compile speed beats exotic analysis.
- Writable from the docs.

How every agent works: Evidence or it isn't true; Goals, commitments and hypotheses stay separate; Read before you write; Fix the cause, not the symptom; Every fix lands with its test; Main stays green; Leave it better; Ask what the maximal version is; Own ordinary decisions; Report crisply; Nothing sensitive in the repo.

Process never costs more than 10% of your time: if it does, skip it and add
`Skipped: <process>, <reason>` to the commit
(`handbook/process/handbook-and-rfcs.md`).

## Commits

- Subject: `area: summary`, lowercase imperative, at most 72 characters,
  e.g. `stdlib: add interval map` or `port(cf2237h): rethink with interval
  map`. A body, when needed, says why, not what.
- Areas: `syntax`, `check` (types and analysis), `codegen`, `interp`,
  `stdlib`, `diag`, `lsp`, `editors`, `fmt`, `lint`, `stress`, `judge`,
  `tools`, `cli`, `docs`, `site`, `design`, `spec` (the normative `spec/`),
  `plan`, `port`, `frictions`, `bench`, `factory` (scripts, briefs,
  retros), `ci`, `deps`, `revert`. An optional scope in parentheses
  narrows it.
- One logical change per commit, each green on its own; a port and its
  friction report go together. Land on `main` with `scripts/land.sh`
  (rebase and push; CI gates every tip and infra keeps main green); never rewrite pushed history without
  Ryan's say-so.
- `scripts/install-hooks.sh` turns on the hooks: `pre-commit` runs the fast
  check (including the public-history scan: only hosts in
  `scripts/public-hosts.txt`, no home paths, nothing from the local
  `.git/info/sensitive-words`), and `commit-msg` checks the subject.
