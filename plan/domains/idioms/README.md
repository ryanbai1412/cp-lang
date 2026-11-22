# idioms

Segment: D. Taste and co-design. Owns: bundled skill, principles, idiom catalog, `research/dryrun`, `corpus/` (ports + new problems), `crates/cpl-judge`, scoreboard, `spec/delta/`. Worklog: [worklog.md](worklog.md). Architecture: [ARCHITECTURE.md](ARCHITECTURE.md).

What idiomatic v0.3 looks like, written down (skill, principles, idiom catalog) and proven (re-ports on every judge, dry runs, blind evals, new problems).

## Covers
**Skill.** The bundled AI skill, the principles, the idiom catalog (which lint and docs build on), blind evals.

**Corpus.** The ports and their C++ originals, judge profiles, scoreboard, bug-replay and revision suites; growing the problem corpus (`meta.json`, `dream.cp`/`spec.cp`), with gaps written up in `spec/delta/`. (Merged corpus-judges + corpus-gen.)

## Improve
**Skill.** Skill v2 on v0.3; idiom catalog; blind contests on the dry-run problems (currently 10/20 on v0.3).

**Corpus.** Re-port the 305 ports idiomatically on v0.3 (5 done), each proven on its judge profile; scoreboard on v0.3; extend the dream/spec hill-climb past 100 problems, targeting thin contest areas; each gap becomes a spec proposal or a conformance case.

## Success criteria
**Skill.** Dry run 20/20 on v0.3; a blind skill-only contest with 0 compiler bugs and 0 non-idiomatic flags from lint.

**Corpus.** 305/305 v0.3 ports accepted under every applicable judge profile; tokens median <= 0.6x C++, none > 1.0x; this corpus is lint's and fmt's false-positive gate; N new problems/week with dream + spec ports; dream-vs-spec gap count trending to 0.

## Ryan's role
**Skill.** Co-designer: the principles and the idiom catalog are Ryan's taste written down; evals check them.

**Corpus.** Reviews a sample of re-ports and dream programs for idiom (they define "idiomatic"); picks problem areas. Judge proof needs no Ryan time.
