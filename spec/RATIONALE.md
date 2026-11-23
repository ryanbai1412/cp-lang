# Why the language is the way it is

An index for readers who want the design and not just the rules, and the
source for the site's in-depth chapters. For each area: the chapter that
gives the overview, the decisions behind it (numbers in `decisions.md`),
and the alternatives that lost, with why. Statuses for Milestone 1 are in
`SUBSET.md`. Older debates are in `research/design-history/`
(`decisions-design.md`, `style-decisions.md`); the library's own
rationale is in `lib/rationale/` (`ranges.md`, `seq.md`, `dp.md`,
`graphs.md`, `search.md`, `trees.md`).

Questions from site or any reader: add them to
`plan/domains/spec/worklog.md`; spec answers within the hour.

## The whole language

- Overview: `core/00-overview.md` (The model, A small program, Layers).
- The goals and design rules every decision is checked against are in
  `AGENTS.md`; the mission and audience in `handbook/product/vision.md`.

## Values and types (`core/02-values-types.md`)

- Moved to [handbook/language/values-and-numbers.md](../handbook/language/values-and-numbers.md).

## Numbers, widths and layout (`core/03-numbers-layout.md`, "The model in one paragraph")

- Moved to [handbook/language/values-and-numbers.md](../handbook/language/values-and-numbers.md).

## Index spaces, positions and keys (`core/04-indices.md`, "The model in one paragraph"; `04a`, `04b`)

- Moved to [handbook/language/indices-and-keys.md](../handbook/language/indices-and-keys.md).

## Ownership and mutation (`core/05-ownership.md`, "The model in six sentences")

- Moved to [handbook/language/ownership.md](../handbook/language/ownership.md).

## Control flow and functions (`core/06-control-functions.md`)

See [`handbook/language/control-and-functions.md`](../handbook/language/control-and-functions.md).

## Sequences (`core/07-sequences.md`, "Key decisions")

See [`handbook/language/sequences.md`](../handbook/language/sequences.md).

## Traits and generics (`core/08-traits-generics.md`)

See [`handbook/language/traits-and-generics.md`](../handbook/language/traits-and-generics.md).

## Contexts, build modes and checks (`core/09-contexts-checks.md`)

- Decisions 100, 145, 148, 243.
- Lost: `@allow(name)` (243; `with lint(name: allow)` is the one spelling).

## Input and output (`core/10-io.md`)

- Decisions 20, 136, 161.
- Lost: `Graph.read` (R10.2.5; edges are records, one read rule).

## Modules and the prelude (`core/11-modules-prelude.md`)

- Decisions 72, 94, 150, 241.
- Lost: an 80-name prelude budget (150; the prelude is exactly the lib
  chapters' main tables, so membership is a rule, not a negotiation).

## Library powers (`core/12-powers.md`)

- Decisions 2, 3, 28, 141, 142, 146.
- Lost: Scala-style by-name parameters and implicit `T` to `() -> T`
  (28; a call site must say when its argument runs). Any power private to
  the stdlib (AGENTS.md, "no privileged stdlib").

## Lowering to C++ (`core/13-lowering.md`)

- Decisions 31, 52.

## The library (`lib/`)

- Range structures: decision 42, `handbook/language/sequences.md`; 316 retires
  `RangeFenwick` and `DualFenwick` for `SegTree(act:)`.
- Containers: 103, 231 (`Map` is the default table; `HashMap` is spelled
  out when hashing is wanted).
- Graphs: 146, 315. Search and DP: 81, 291. Flows: 112.

## Overviews

Every core chapter opens with a short overview ("The model in one
paragraph", or chapter 5's "The model in six sentences"), and so does
every `lib/` chapter; the site's in-depth chapters start there and
follow the rule sections for detail.
