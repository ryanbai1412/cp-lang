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
  `AGENTS.md`; the mission and audience in `plan/product/VISION.md`.

## Values and types (`core/02-values-types.md`)

- Decisions 1, 127, 128, 132, 134, 140, 157, 159.
- Lost: method overloading (127; a name has one meaning, so errors and
  completion stay exact). Custom destructuring (132; views are methods
  returning enums, so patterns stay core-only). Members that compute
  without `()` (140; parentheses mean work, so `g.transpose()` looks like
  the \(O(n)\) it is).

## Numbers, widths and layout (`core/03-numbers-layout.md`, "The model in one paragraph")

- Decisions 49, 100, 148, 149, 246.
- Lost: manual widths (`u32` fields) everywhere (149; the range or domain
  you already wrote gives the width, and each section names its rejected
  alternative). Implicit widening of arrays (246; only scalars widen, an
  array conversion is \(O(n)\) and spelled `xs.map(...)`). Checks on in
  release (148; costs judge time; a trapping program gives a wrong answer,
  never undefined behaviour). Path lengths in the weight's range (315).

## Index spaces, positions and keys (`core/04-indices.md`, "The model in one paragraph"; `04a`, `04b`)

- Decisions 2, 137, 143, 162, 164, 246, 289, 291.
- Lost: every range as positions with a separate `r.cuts` (137; a cleaner
  model, but `for j in lo..i: dp[i] min= dp[j] + cost(j..i)` stops
  type-checking). Slices re-keyed from 0 (143; `04a-identity-vs-offset.md`
  walks every option against real code: an index found in a slice must
  mean the same position outside it). Positional `r[k]` on a range (289,
  Ryan: "r[3] should be equal to 3"; position is `r.nth(k)`). Names
  `.n`/`.p` (137; read as count and parent).

## Ownership and mutation (`core/05-ownership.md`, "The model in six sentences")

- Decisions 5, 126, 128, 141, 144.
- Lost: implicit deep copies (128; `.clone()` is the copy, returns move,
  `let b = a` is a move at last use and otherwise an error). Borrow modes
  in types and lifetimes (141; a value is owned or lent, the mode is not
  part of the type, and `lends` appears only on fn signatures: five rules
  instead of a borrow checker's vocabulary). `mut fn` (144, 5.10).
  Pull-style library iterators (126; generators are push-only, so
  `break`, `return` and labelled loops work in a plain `for`).
- The chapter's exercises (R1 to R12) are the in-depth material; each has
  a conformance case.

## Control flow and functions (`core/06-control-functions.md`)

- Decisions 60, 98, 125, 126, 127, 129, 130.
- Lost: `_by` twins (60; one `by:` option). `enumerate` (60; `.items`).
  `++` and unary `+` (98). Compound assignment as an expression (129).
  Mixed-direction comparison chains (130).

## Sequences (`core/07-sequences.md`, "Key decisions")

- Decisions 53, 142, 143.
- Lost: free-function folds and sorts (53; methods, so completion finds
  them). One iterator trait (142; six capabilities say exactly what a
  view supports, so an \(O(n)\) read never hides behind `s[k]`).

## Traits and generics (`core/08-traits-generics.md`)

- Decisions 3, 132, 146.
- Lost: arcs stored in a side table (146; rarely worth it, `Next` carries
  arc capabilities instead).

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

- Range structures: decision 42, `lib/rationale/ranges.md`; 316 retires
  `RangeFenwick` and `DualFenwick` for `SegTree(act:)`.
- Containers: 103, 231 (`Map` is the default table; `HashMap` is spelled
  out when hashing is wanted).
- Graphs: 146, 315. Search and DP: 81, 291. Flows: 112.

## Overviews

Every core chapter opens with a short overview ("The model in one
paragraph", or chapter 5's "The model in six sentences"), and so does
every `lib/` chapter; the site's in-depth chapters start there and
follow the rule sections for detail.
