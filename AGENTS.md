# AGENTS.md

Read this first if you are working on CP Lang, agent or person: what we are
building, what it owes the contestant, and the rules every change follows.

The handbook (`handbook/README.md`) is becoming the one home for how we
work: read `handbook/process/handbook-and-rfcs.md` for how it is kept true,
when to write an RFC, and the escape hatch (process over 10% of your time
may be skipped). Its sections land one at a time; until a section lands, the
files below remain authoritative.

Start with `handbook/product/`: `vision.md` (who CP Lang is for and why),
`journeys.md` (the journeys we owe them) and `milestones.md` (the
milestone every lane works toward). Product docs say what users get, `spec/`
says what the language is. When choosing what to do next, unblock a step of
the milestone's acceptance scenario first.

Then read, in order:
0. `plan/STATUS.md` (generated: every domain's metrics and open questions) and `spec/README.md`: where the 0.3.0 work stands today,
   how the spec and corpus fit together, the checks to run, and what is open.
   `spec/` (`spec/core/`, `spec/lib/`, `spec/tools/`) and the stdlib in
   `std/` state the language precisely; where anything else disagrees with
   them, they win. Once `handbook/language/` lands, it leads: it says what
   the design means, the spec states it precisely, and the two change
   together.
1. `plan/OPERATING.md`: settled decisions and the loop we work in.
2. `plan/DOMAINS.md`: the domains (one domain = one lane = one owner);
   find yours, then read `plan/domains/<domain>/` and `plan/QUESTIONS.md`.
3. `plan/ARCHITECTURE.md`: compiler, IR, interpreter, and crate architecture.
4. `decisions.md`: every design call, its reason and how to reverse it.
5. `REPO.md`: where everything lives.
6. `handbook/process/porting.md`: if you port solutions, the bar and the review.

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
