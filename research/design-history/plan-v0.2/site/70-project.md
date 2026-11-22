# The project (`/status`, `/design`, `/problems`, ...)

Status: proposed. Everything here is generated from repo files, so it
can't drift.

## `/status`: what works today

- The beta statement, one paragraph.
- **Feature status**: for each spec chapter and library family, the
  share of its snippets that `run`, `check` or are `spec` only, from the
  snippet gate's results. Links to `spec/delta/`.
- **Domains**: one card per area from `plan/DOMAINS.md` (north star,
  where it stands, next).
- The compiler version the playground runs and the spec version the
  docs describe.

## `/design`: why the language is this way

- The mission and the six goals, from `AGENTS.md`, rewritten for users
  (hand-written, reviewed against the source).
- The design rules, each with a one-paragraph example.
- "Why not just C++?" and "Why not Python/Rust/Kotlin?".
- What's out of scope, from `AGENTS.md`.

## `/design/decisions`: the decisions index

`decisions.md` parsed into one entry per decision (number, title,
decision, reason, how to reverse), each with an anchor
(`/design/decisions#157`), a search box and tag filters. Spec pages link
decision numbers here.

## `/problems`: real problems

The corpus gallery, from `corpus/index.json` and
`corpus/problems/*/meta.json` *(verified layout)*:

- A list with filters (tags, difficulty, judge) and per problem: name,
  judge link, tags, line and token counts (C++ vs CP Lang).
- A problem page: the accepted C++ beside `spec.cp`, the notes (decision
  numbers linked), and the judge's statement link. Never the statement
  text.
- Only records with `spec` status `reviewed` or `gold` appear (today all
  100 hill-climb records are `draft`, *verified*, so the gallery starts
  with the gold exemplars and fills as records are reviewed).
- `spec.cp` carries a status badge; most `check` but don't run yet.
- `dream.cp` is not shown.
- Needs Ryan's and Andrew's OK to publish their submissions.

## `/performance`

Generated from `bench/corpus-p95-results.json` and the latency audit:
runtime median, p90, p95, worst; peak memory; largest emitted file vs
the 64 KB limit and 32 KB target; front-end latency vs the 500 ms budget;
the 20 slowest ports with links; the run's commit, compiler and
machine. Today's run *(verified)*: 343 ports, median 0.98x, p95 2.61x
against the 1.1x target, largest file 33.6 KB. The page says "target
not met" until it is.

## `/releases`: release notes

One page per release from a `CHANGELOG.md` we'd start keeping (one
section per version: language, library, tools, breaking changes with
codemods), an RSS feed at `/releases.xml`, and a "what's new in 0.x"
summary on the latest. The landing footer links the latest.

## `/contributing`

How to build and run the gate, commit conventions (from `AGENTS.md`),
where to start (domains with open next steps), how to report a docs bug.
