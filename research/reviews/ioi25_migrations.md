# ioi25_migrations

Score: 4/5
Dream: 4/5

Re-reviewed cold on fbed8d10d against the statement (IOI 2025 Day 2, Migrations) and
`dream.cp`; the friction report was read only after scoring. Since ba1cf559f the gates judge the
`@grader(two_phase)` dream through `dream.tests/grader.cpp`, so `port.cp` is gone and the dream is
the package's only program: both lines score it. The first review's port score (2/5,
provisional) applied to the deleted port.

Dream 4/5. The header now explains the five phases, what each of the last three symbols names,
and why `None` padding is never spoken, so the case tables can be checked against it. The first
run's state is one `struct Sender` with an `arrive` method, not eleven top-level `var`s reset by
hand. Every candidate and end is a `Site`, and the endgame lists are `?Site` padded with `None`.
Each endgame step stores the lists its symbol leaves; no fn both sets state and returns it. At
`--max` the dream takes 10-20 ms and 3.17-4.09 MB against orig.cpp's 3.97-4.22 MB.

## Checklist failures

None. The friction report now exists, with the measurement against orig.cpp at `--max`, and
the header has its `// limits:` line.

## Owed rewrites

- `dream.cp:22`, `struct Phase(start: Int, ...)`: a phase starts at a site, so `start: Site`
  would retire the `at.ord` comparisons at `dream.cp:144-156`. Route: the author (minor).

## Language gaps

Reported: struct fields whose type is inferred from a default are emitted as `void` (the three
annotations), no `Site` at the grader boundary, the placeholder `var sender = Sender(0, [])`
(no per-run grader state), L0507 over a stateful call, the earlier grader-module and missing-name
gaps, and the `List<0..=4>` storage widening.

## Dream notes

- `dream.cp:51-76, 90-101`: `cpl fmt` puts the 24- and 10-row case tables one pair per line;
  already sent to lint-fmt as a fmt friction (research/frictions/fmt-corpus.md).

Addressed (minor): `struct Phase(start: Site, ...)`. `Sender.arrive` compares
`plan[next_phase].start == at` and `longest_path` extends with `Site(next)..=phase.start`. Only
`heard`'s symbol window and `next` still take `.ord`, since both count positions in `s`.
