# CP Lang 0.3.0 specification

This tree is the language and library we intend. It is normative: where an
older document (`design/spec/`, `plan/1x-*.md`) disagrees, this tree wins,
and `decisions.md` records why each call was made and how to reverse it.
Implementation follows and is measured against these files; what is not
built yet is listed in `delta/`.

Status: being assembled into 0.3.0 (2026-11-21). Chapters marked *draft*
are written; *planned* ones are placeholders in this index only.

## Reading order

Core language, read in order:

1. `core/00-overview.md`: the model in one page, the trust tiers, the
   build modes. *draft*
2. `core/01-lexical-grammar.md`: tokens, layout, the grammar. *draft*
3. `core/02-values-types.md`: values, records, structs, enums, optionals,
   generics. *draft*
4. `core/03-numbers-layout.md`: the four number types, ranges, storage
   widths, niches, layout, overflow. *draft (aligned)*
5. `core/04-indices.md`, `04a-identity-vs-offset.md`, `04b-keys.md`:
   domains, positions and cuts, ranges, identity everywhere. *draft
   (aligned)*
6. `core/05-ownership.md`: owned and lent values, `mut`, lends, closures,
   generators, `@cache`, exercises. *draft (aligned, authoritative)*
7. `core/06-control-functions.md`: statements, loops, functions,
   closures, generators. *draft*
8. `core/07-sequences.md`: generators, views, the six capabilities.
   *draft (aligned)*
9. `core/08-traits-generics.md`: traits, associated types, aliases, laws,
   row types. *draft*
10. `core/09-contexts-checks.md`: contexts, `with`, what each build
    checks. *draft*
11. `core/10-io.md`: `read<T>`, printing, multitest, interaction.
    *draft*
12. `core/11-modules-prelude.md`: modules, pins, the prelude. *draft*
13. `core/12-powers.md`: what user libraries can define (no stdlib
    privilege). *draft*
14. `core/13-lowering.md`: how each form becomes C++. *draft*

Library design, one chapter per family (the stubs in `std/` are the
signatures and costs; these chapters are the model and the examples):

- `lib/search-dp.md` *aligned*, `lib/graphs-trees.md` *aligned*,
  `lib/ops-ranges.md` *aligned*, `lib/sequence-patterns.md` *aligned*.
- `lib/containers.md`, `lib/strings.md`, `lib/math.md`, `lib/flows.md`,
  `lib/geometry.md`: family surveys and APIs from the stdlib design
  round, *draft*, each headed by the 0.3 rules that override it.
- `lib/random-stress.md`: `Rng`, generators of typed inputs, brutes and
  checkers. *draft*
- `lib/rationale/`: the pre-round surveys for search, DP, graphs, trees,
  ranges and sequences, superseded by the chapters above.

Tools, one contract per tool (`tools/`): CLI, judge profiles, stress and
shrinking, explain, trace and debugger, fmt and lint, LSP, bundler.
Errors and lints: `diagnostics.md`. Examples that must and must not
compile: `conformance/`. What 0.2.x does not do yet: `delta/`.

## Checking

- `std/` holds every stdlib module as the spec wants it. Bodies are
  `todo` with a doc comment and a cost; `scripts/spec-stub.py --check`
  keeps it so. A file here shadows `stdlib/` (`CPL_STD` is searched
  first).
- `scripts/spec-check.sh` holds the gold dreams (`GOLD.txt`, `gold/`) and
  the port dreams to the stubs. A failing dream is either a spec gap (add
  the signature or the power) or a dream that ignores a decision (fix the
  dream). Never weaken a dream to make it pass.
- `delta/parser-gaps.md` lists decided syntax the front end does not
  accept yet.
