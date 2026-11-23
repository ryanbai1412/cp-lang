# The CP Lang handbook

CP Lang is a programming language for competitive programming: you write
a solution in it, check and test it locally, and submit the single C++17
file it generates. This handbook describes how the project works today:
what we are building and for whom, what we value, how work flows, and why
the language and its tools are shaped the way they are. It is written for
someone who has never seen the project, whether a new contributor, a
curious reader or a freshly started agent.

## How it is organised

- **Product** says who CP Lang is for, the journeys we owe them, and the
  milestones we are working toward. Start here to understand why anything
  matters.
- **Principles** are the goals every change is judged against and the
  rules that settle design questions.
- **Language** explains why the language is shaped the way it is. The
  spec in `spec/` then states the same design exactly, rule by rule.
- **Design** explains how the compiler, editor support and the other tools
  are built, and which component owns each fact.
- **Process** covers how work flows from an idea to a release, who decides
  what, and how quality is kept high. The process budget is part of it:
  process takes under 10% of anyone's time, and anyone may skip a process
  that costs more.
- **Guidelines** explain how to do one kind of work well, such as designing
  a std API or writing an error message, each with an exhaustive checklist.
- **Domains** has one charter per area of the project: what it owns, what
  good looks like, and every quality it tracks.

Each topic follows the same pattern: a readable document that gives the
picture, paired with an exhaustive reference (the spec, a checklist, a
quality register) that tests and lints cite. See `guidelines/writing.md`.

## Where to begin

- **New to the project:** `product/vision.md`, then `principles.md`.
- **Starting work in a domain:** `domains/README.md`, then your domain's
  charter, then `process/factory.md` and `process/change.md`.
- **Changing the language:** the matching document in `language/`, then
  the spec chapter it leads; both change together.
- **Proposing a big change:** `process/handbook-and-rfcs.md`.

---

# Reference

## Where each kind of fact lives

Every fact has one authoritative home. Other documents explain it, link to
it or are generated from it; they don't restate it.

| Kind of fact | Home |
|---|---|
| Who it's for, what it should feel like, milestone bars | `handbook/product/` |
| Values and design rules | `handbook/principles.md` |
| Why the language is shaped this way: the model, the decisions, the rejected alternatives | `handbook/language/` |
| Exactly what the language does, edge cases included | `spec/`, with a conformance case per rule |
| How the toolchain is built, its contracts and invariants | `handbook/design/` |
| How work happens, who owns what, who decides | `handbook/process/`, `handbook/domains/` |
| What we're working on now, current numbers, blockers | `plan/` and generated metrics |
| Why a major change was made | `handbook/rfcs/`, `decisions.md` |
| What the code actually does | the code, checked by tests against the above |

**Language design and spec move together.** The handbook's language docs
lead: they say what we mean. The spec states the same design precisely. A
language change, by an RFC or a direct edit, updates both in the same
change, and an edit made to the spec first also updates the matching
handbook section. If the two disagree, that's a consistency bug for the spec
owner to resolve: usually the spec is fixed to match the intent, but
sometimes writing the spec exposed a case where the intent was wrong, and
then the handbook is fixed.

**Durable knowledge here, numbers elsewhere.** A charter says *the compiler
must pass every conformance case*. How many pass today is a generated
number. *Fix the remaining failures* is in the domain's plan.

## Common tasks

| I want to… | Read |
|---|---|
| make any change | `process/change.md`, then my domain's charter |
| change the language | `language/<area>.md`, the spec chapter it leads, `process/change.md` |
| add or change a std API | `guidelines/std-api.md`, `guidelines/std-docs.md` |
| write or improve an error message | `guidelines/errors.md` |
| change the parser, checker, IR, VM or codegen | `guidelines/compiler-changes.md`, `-checklist.md` | How to fix the compiler so the fix holds for every program; checklist `compiler-changes/R1`..`R11` | compiler |
| add a test suite, or know which test to write | `process/quality.md` |
| add a conformance case or change a spec rule | `guidelines/conformance.md`, `guidelines/spec-rules.md` |
| land a commit, or understand a red CI | `process/landing-and-ci.md` |
| make a decision outside my domain | `process/ownership.md` |
| propose a major change | `process/handbook-and-rfcs.md` |
| find where code lives | `design/repo.md`, `design/overview.md` |

## How to use it

- **One owner per document,** named at the top. Anyone may fix a document
  that is wrong or out of date. The owner keeps it coherent. Changing a
  contract another domain relies on needs that domain's owner.
- **One topic per document,** using whichever of these sections apply:
  *Goals*, *Process*, *Guidelines*, *Checklist*, *Measures*.
- **Process never costs more than 10% of your time.** If it does, skip it,
  whatever it is, and note `Skipped: <process>, <reason>` in the commit.
  Skipped tests or gates are picked up in the hourly post-land review and
  fixed forward. A process that keeps getting skipped is fixed or deleted.
  See `process/handbook-and-rfcs.md`.

## Contents

### Start here

| File | What it answers | Owner | Absorbs |
|---|---|---|---|
| `principles.md` | Mission, goals and their tests, design rules, how every agent works | coordinator | The body of `AGENTS.md`, which shrinks to a short version plus pointers here |
| `glossary.md` | Project terms: domain, lane, port, corpus, ratchet, edge, conformance case… | coordinator | Terms scattered across plan files (language terms stay in `spec/GLOSSARY.md`) |

### Product: what we're building, and for whom

| File | What it answers | Owner | Absorbs |
|---|---|---|---|
| `product/vision.md` | Who CP Lang is for, what it should feel like, what success looks like | coordinator | `handbook/product/vision.md` |
| `product/journeys.md` | The user journeys end to end: install, first program, learning through docs and lints, a contest, debugging, the editor, submitting | coordinator | `handbook/product/journeys.md` |
| `product/milestones.md` | What a milestone is, and each one's bar per journey and per domain: Milestone 1 (edge), Milestone 2 (stable) | coordinator | `handbook/product/milestones.md`, `MILESTONE-2.md`, `README.md` |

How far along each milestone is, and what blocks it, stays in `plan/`.

### Process: who does what, and when

| File | What it answers | Owner | Absorbs |
|---|---|---|---|
| `process/handbook-and-rfcs.md` | Keeping the handbook true, when to write an RFC, how one is decided and later superseded, the escape hatch | coordinator | New (Handbook 0001) |
| `process/change.md` | How any change flows: observe, find the right layer, decide, update the design and spec, implement with tests, verify, update docs, land. Small fixes collapse most steps | coordinator | Scattered rules in `AGENTS.md` and `plan/OPERATING.md` |
| `process/ownership.md` | Roles (domain owner, milestone owner, task owner), what each may decide, who may edit vs who may change a contract, what comes to Ryan | coordinator | `plan/DOMAINS.md` intro, decision rules in `AGENTS.md` |
| `process/factory.md` | How the factory runs: lanes, the coordinator, hourly status, unblocking yourself | coordinator | `plan/OPERATING.md`, parts of `plan/agent-context.md` |
| `process/quality.md` | How we ensure quality: the toolkit, quality registers, which test to write for which change, how suites are measured, accountability | tech-debt | New, plus test rules in `AGENTS.md` and `handbook/design/architecture.md` A6 |
| `process/landing-and-ci.md` | How commits land, what runs where, time budgets, who keeps main green | infra | `plan/domains/infra/design/gate-tiers.md` |
| `process/release.md` | Channels (edge, stable), versioning, what makes a commit shippable, installers, promoting a release | infra | `plan/domains/infra/RELEASE.md` |
| `process/porting.md` | Porting accepted solutions into the corpus: the bar and the review | idioms | `plan/PORTING.md` |

### Guidelines: how to do one kind of work well

| File | What it answers | Owner |
|---|---|---|
| `guidelines/std-api.md` | Naming lexicon, return conventions (`?T`), named options, keys and indices, method vs free form, user journeys per type, review checklist | stdlib |
| `guidelines/std-docs.md` | Required sections, trait bounds, cost, examples that run, readability; checklist | stdlib |
| `guidelines/compiler-changes.md` | Fix the class where the spec rule lives, regression with every fix, one program for two engines, bounds, no error inside std, migrate existing programs; checklist | compiler |
| `guidelines/errors.md` | One error per root cause, at the mistake, naming the rule and a fix that compiles; checklist | diagnostics |
| `guidelines/lints.md` | When a lint is warranted, how it teaches, configuration, fix quality | lint-fmt |
| `guidelines/performance.md` | Matched-algorithm comparison, runtime and size bars (including the tuned max), behavior never changes | perf |
| `guidelines/conformance.md` | What makes a conformance case worth having: one rule, expected result from the spec, known gaps that fail for their reason, generated tables, engine agreement; checklist | spec |
| `guidelines/spec-rules.md` | Writing a spec rule: one general rule, one spelling picked by type, rule ids, the three questions, chapters agree with std; checklist | spec |
| `guidelines/writing.md` | Writing for an outsider: docs, commit messages, reports | coordinator |
| `guidelines/idiomatic-code.md`, `guidelines/idiomatic-code-checklist.md` | How ports, dreams, skill pages and examples read: editorial first, std over hand-rolled, each fact once, constraints as types, one spelling; rules `idiomatic-code/R1`.. | idioms |
| `guidelines/problem-packages.md`, `guidelines/problem-packages-checklist.md` | Tests as generators and seeds, answers from accepted solutions, checkers, interactors, max inputs, one package format; rules `problem-packages/R1`.. | idioms |
| `guidelines/site.md`, `guidelines/site-checklist.md` | Site pages: every program checked, one source per fact, generated pages, structured reference entries, real links, editors through the language server, phone and weight; rules `site/R1`.. | site |

### Language: why the language is shaped this way

One design doc per area: the problem it solves, the model a contestant
holds in their head, the key decisions and the alternatives that lost, with
worked examples. These are also the source of the site's in-depth chapters.

| File | Spec chapters it leads | Owner |
|---|---|---|
| `language/overview.md` | `core/00` | spec |
| `language/values-and-numbers.md` | `core/02`, `03` | spec |
| `language/indices-and-keys.md` | `core/04`, `04a`, `04b` | spec |
| `language/ownership.md` | `core/05` | spec |
| `language/control-and-functions.md` | `core/06` | spec |
| `language/sequences.md` | `core/07`, `lib/sequence-patterns.md` | spec |
| `language/traits-and-generics.md` | `core/08` | spec |
| `language/checks-and-io.md` | `core/09`, `10` | spec |
| `language/modules-and-powers.md` | `core/11`, `12` | spec |
| `language/std.md` | `lib/*` | stdlib |

They absorb `spec/RATIONALE.md`, `spec/lib/rationale/` and the design
reasoning spread through `decisions.md`.

### Design: why the toolchain is shaped this way

| File | What it answers | Owner | Absorbs |
|---|---|---|---|
| `design/overview.md` | Text to C++: the pipeline, the crates, which layer may depend on which | tech-debt | `plan/ARCHITECTURE.md` Part A |
| `design/architecture.md` | The target architecture in full: what shapes it, each layer, the crate map the gate checks, testing, the repository | tech-debt | `plan/ARCHITECTURE.md` Part A |
| `design/sources-of-truth.md` | One owner per fact and one entry point per job in the code: what's generated from what, what must never be written twice, and the detector for each | tech-debt | `plan/rearchitecture.md` B2 |
| `design/compiler.md` | Syntax, names, types, IR, facts, VM, codegen: each part's job, data, invariants, and what it must never do | compiler | `plan/domains/compiler/ARCHITECTURE.md` |
| `design/codegen.md` | The C++ emitter: what each part decides, the one copy and effect analysis, its invariants and modules | perf | `plan/domains/perf/ARCHITECTURE.md` |
| `design/editor.md` | The query database, LSP and playground sharing one analysis | tools | `plan/domains/tools/ARCHITECTURE.md` |
| `design/lint-fmt.md` | The linter and formatter: one analysis, rules as records, a token-preserving formatter | lint-fmt | `plan/domains/lint-fmt/ARCHITECTURE.md` |
| `design/std-mechanisms.md` | How std gets its powers without privilege | stdlib | `plan/domains/stdlib/library-domains.md` |
| `design/debugger.md` | Tracing, replay, stress, the debugger | debugger | the debugger plan's architecture notes |
| `design/repo.md` | File structure, toolchain, scripts | infra | `REPO.md`, `plan/LAYOUT.md` |
| `design/infra.md` | Landing, CI, releases and installers: each part, the path of a commit to a user, the invariants | infra | `plan/domains/infra/ARCHITECTURE.md` |

### Domains: one charter each

`domains/README.md` says what a domain is and lists them. `domains/<name>.md`
for compiler, perf, diagnostics, stdlib, spec, idioms, lint-fmt, tools,
debugger, site, infra and tech-debt, each with the same sections:

- **Mission:** one paragraph.
- **Scope:** what it owns (paths, crates, commands) and what it doesn't.
- **Interfaces:** what it provides to other domains and needs from them.
- **Quality register:** every quality attribute the domain must be high on.
  Each has a metric and the mechanism that enforces it (a suite, a lint, a
  ratchet, a re-architecture), or the process used where no mechanism can
  decide. Targets and current numbers are generated into `plan/`.
- **Suites and tools:** what exists, what each catches, how to run it.

The domain's `PLAN.md`, `metrics.toml` and `worklog.md` stay in
`plan/domains/<name>/`.

### Records

| Place | Holds |
|---|---|
| `rfcs/NNNN-name.md` | Major changes as decided. A decision can be superseded by a later RFC; the record itself never changes |
| `../decisions.md` | The log of smaller decisions. Searchable, not required reading |

## What happens to existing files

- `AGENTS.md` keeps a short version of the mission and rules, since every
  agent loads it automatically, and points here. Its rule that `spec/` is
  the only normative source becomes the rule above.
- `plan/OPERATING.md`, `plan/DOMAINS.md`, `plan/LAYOUT.md`, `REPO.md`,
  `plan/PORTING.md` and `handbook/design/architecture.md` are folded in.
- `plan/product/` has moved to `product/`; `BLOCKERS.md` and `notes.md` stay
  in `plan/product/`.
- `plan/agent-context.md` (Ryan's own words) moves to
  `research/design-history/` as background; its durable points go into
  `principles.md` and `product/`.
- `plan/STATUS.md` and `plan/QUESTIONS.md` stay.

## Keeping it true

- Each document's *Describes* line lists the files and commands it covers.
  A check flags a document whose paths no longer exist.
- Code examples in the handbook compile and run in CI, and references to std
  items, files and other documents are checked.
- Facts that can be generated from code or the spec (lists of lints,
  metrics, crates) are generated, not written.
- When a change makes a document wrong, the same commit fixes it.
- A new agent misreading a document is a failure of the document.

## Changes

- 2026-11-23: first draft.
