# corpus

Segment: D. Taste and co-design. Owns: `corpus/` (ports + new problems), `crates/cpl-judge`, scoreboard, `spec/delta/`. Worklog: [worklog.md](worklog.md).

## Covers
The ports and their C++ originals, judge profiles, scoreboard, bug-replay and revision suites; growing the problem corpus (`meta.json`, `dream.cp`/`spec.cp`), with gaps written up in `spec/delta/`. (Merged corpus-judges + corpus-gen.)

## Improve
Re-port the 305 ports idiomatically on v0.3 (5 done), each proven on its judge profile; scoreboard on v0.3; extend the dream/spec hill-climb past 100 problems, targeting thin contest areas; each gap becomes a spec proposal or a conformance case.

## Success criteria
305/305 v0.3 ports accepted under every applicable judge profile; tokens median <= 0.6x C++, none > 1.0x; this corpus is lint's and fmt's false-positive gate; N new problems/week with dream + spec ports; dream-vs-spec gap count trending to 0.

## Ryan's role
Reviews a sample of re-ports and dream programs for idiom (they define "idiomatic"); picks problem areas. Judge proof needs no Ryan time.
