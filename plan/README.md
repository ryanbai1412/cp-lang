# The plan

This directory is the single source of truth for what we are building. It is
written to be self-contained: an agent or person with only these files (plus
`agent-context.md` and the corpus) should be able to build the same thing.
Older design docs have been folded in here and moved to `archive/` (kept for
history; when they disagree with `plan/`, `plan/` wins).

Reading order for a new agent: `agent-context.md`, then 00, 1.1, 1.8, the
rest of 1.x, then the parts you're building. How the compiler is
implemented is up to the builder; the plan specifies behaviour, guarantees
and quality bars.
Open items: `../decisions.md`, `18-key-features.md`,
`../research/RECOMMENDATIONS.md`.

Each document states its status at the top. "Decided" means the user agreed;
"Proposed" means an agent made the call and it hasn't been reviewed yet.

## Documents

0. [Vision and goals](00-vision.md)
1. Language
   - [1.1 Overview and a tour](10-language-tour.md)
   - [1.2 Values, types and representation](11-values-and-types.md)
   - [1.3 Domains, sequences and views](12-domains-and-sequences.md)
   - [1.4 Abstraction: traits, operation values, generics](13-abstraction.md)
   - [1.5 State: ownership, borrowing, rollback](14-state.md)
   - [1.6 Program structure, modules and libraries](15-program-structure.md)
   - [1.7 Input, output, interaction](16-io.md)
   - [1.8 Syntax and style reference](17-syntax.md)
   - [1.9 Key features: input types, contracts, compile-time evaluation, REPL (proposed)](18-key-features.md)
   - [1.10 Bounded indices and lends: one mechanism for `domain`, `Str`, `Grid` and `Perm`](19-index-and-lends.md)
2. [Standard library](20-stdlib.md)
3. [Compiler and runtime](30-compiler.md)
4. [Tooling and the `cp` command](40-tooling.md)
   - [4b. Developer tools: formatter, linter, profiler, coverage, bench, docs, versions](45-dev-tools.md)
5. [Debugging: traces, snapshots, stress testing](50-debugging.md)
6. [Editor support and the language server](60-editor.md)
7. [Judges and submission](70-judges.md)
8. [Validation: tests, corpus, benchmarks](80-validation.md)
9. [Documentation site, learning resources and playground](90-docs-and-playground.md)
   - [9b. Website rebuild: the detailed plan](site/README.md)
10. [Migration from C++](a0-migration.md)
11. [Distribution and versioning](b0-distribution.md)
12. [LLM-friendliness](c0-llm.md)
13. [Roadmap and milestones](d0-roadmap.md)

Plus [agent-context.md](agent-context.md): everything a new agent needs to
know about how this design came about, especially the user's feedback.
