# RFC 0002: Product surfaces and use cases

| | |
|---|---|
| Owner | coordinator |
| Status | Draft: under discussion with Ryan |
| Ryan to review | yes |
| Changes | `handbook/product/`, new `handbook/process/product-design.md`, `handbook/guidelines/writing-product.md`, the domain charters |

## Summary

We decide what to build top down. Every part of CP Lang a user touches is a
*surface*, grouped into *areas*. For each surface we write how it should feel
and list every *use case* it must serve, each a stable need with an ID, a
chosen design and the evidence that the design works. We build this model
once, as completely as we can, then keep refining it: each omission found is
added. Gaps show up as listed, unbuilt use cases rather than as user
reports. Coverage is half the job; the other half is finding the design
that makes each use case one smooth call.

## Problem

Today we find frictions one at a time. A trial turns up `.find()` with no
index, `max()` sometimes returning `T` and sometimes `?T`, and hover missing
`by:`. Each gets fixed, but nothing listed them in advance, so the next trial
finds the next batch. We also spend little time deciding what the ideal
experience is before building it.

## Proposal

### The hierarchy

1. **Journey:** an end-to-end task a contestant does, crossing surfaces:
   "I have WA and want to know why". Today's J1 to J10 in
   `handbook/product/journeys.md`, refined into workflows. Each has an
   owner, an ideal walkthrough and end-to-end acceptance cases. Few and not
   exhaustive. The top one is M1 itself: a friend learns CP Lang, solves,
   tests, debugs and submits in a real contest.
2. **Area:** a group of surfaces with one README that summarizes them in prose.
3. **Surface:** one thing a user touches, with one owner and a short doc.
4. **Use case:** one need, exhaustively listed per surface, with a stable ID.
5. **Acceptance case:** the evidence that the design meets the need.

Journeys catch broken compositions (stress finds a counterexample, but
tracing it takes three commands). Surfaces give fine-grained coverage.

### What it looks like

```text
handbook/product/
  vision.md  journeys.md  milestones.md
  std/
    README.md               the area: one paragraph per surface
    sequences.md            the surface doc
    sequences-usecases.md   its use cases
    sorted.md  sorted-usecases.md
    ...
  cli/       README.md  check.md  run.md  stress.md  trace.md ...
  editor/    README.md  completion.md  hover.md  goto.md ...
  site/      README.md  learn.md  reference.md  playground.md ...
  debugger/  README.md  ...
  release/   README.md  install.md  channels.md ...
```

Areas and their surfaces:

| Area | Surfaces | Owner |
|---|---|---|
| std | sequences, sorted containers, range structures, graphs, trees, strings, math, grids, io | stdlib |
| cli | check, run, test, build, stress, trace, explain, fmt, fix, new | tools |
| editor | completion, hover, signature help, go to definition, inlay hints, rename, quick fixes, install per editor | tools |
| site | landing, learn, reference, playground, practice | site |
| debugger | traces, breakpoints, replay | debugger |
| performance | explain costs, profiles, judge profiles | perf |
| release | install, channels, upgrades | infra |
| language | recursion and closures, indices and domains, DP state, mutation and views, multitest | spec |

The language area doesn't restate `spec/`. Its use cases are programming
experiences, such as "the DP gains a dimension", and link to the spec rules
and corpus programs behind them.

An area README, `std/README.md`, opens like this:

> **Sequences** are what you reach for first: arrays, lists, slices and the
> generators over them. You should be able to search, sort, scan and group
> one without a loop and without losing track of indices.

A use case, in `std/sequences-usecases.md`:

```text
### seq/U7: position of the first element that matches
Need: you are scanning for where a property starts to hold, and need its position, not the value.
Design: a.find_index(p) -> ?I, where I is a's index domain
Milestone: M1-critical. Status: gap
Evidence: program seq-u7.cp; completion offers find_index after `a.fi`
```

The ID names the need, so the design can change without renumbering. The
design links to the spec for exact semantics and doesn't restate them.
Where context changes the best expression, a use case can name more than
one design.

**Granularity:** two situations are one use case when the contestant has the
same intent and the same design should serve both; otherwise they are two.
First match in any sequence and the transition point of a monotone
predicate are two use cases. First match in an array and in a generator are
one.

**Dimensions:** each surface doc lists its dimensions of variation, for
sequences the operation, the shape (array, slice, generator), index
semantics, mutability, empty input, lazy or eager, and cost. Use cases are
found by walking the combinations, not by brainstorming alone.

### How we design a surface

1. **Understand the need:** the journeys it serves, how the 877 corpus ports
   use it, STL, Python, Rust and AtCoder Library equivalents, textbook topic
   lists, and trial reports.
2. **Dream:** write the ideal code or interaction for representative
   problems before looking at what CP Lang does today.
3. **Design the model:** find the smallest set of abstractions that
   serves the use cases. Consider the status quo and real alternatives
   seriously; don't invent a second option to tick a box.
4. **Challenge:** try the design against every use case and many real
   programs. The reviewer asks how close it is to the dream, whether one
   general mechanism could replace several, why it beats C++, Python and
   Rust, where it gets awkward or slow, and how it composes with the rest
   of CP Lang. New needs found here go back into the list.
5. **Commit:** the use cases, their designs and their acceptance cases.

Depth depends on uncertainty, impact, how costly the decision is to
reverse, and how many surfaces it touches. The levels are heuristics:

| Level | Example | Needs |
|---|---|---|
| A | A bug, or building a listed use case whose design is settled | Nothing extra |
| B | A new use case, or a changed design | Its use-case entry |
| C | A new surface, or a model across surfaces (indices) | The five steps and an independent review by another lane. Ryan reviews only what changes CP Lang's character or settles an open question of taste |

### Evidence

An acceptance case is one of:

- **program:** a CP Lang file and its expected output, run in CI.
- **integration:** an editor, CLI or browser fixture, such as completion
  offering `find_index` after `a.fi`.
- **comparison:** the design side by side with its alternatives on real
  programs, recorded once in the surface doc.
- **trial:** a friction from a user trial, matched to a use case.

Programs and integrations prove behaviour; comparisons and trials are how
we judge whether the design is good.

### Keeping it true

- Every acceptance case of kind program or integration runs in CI.
- Every public std fn names the use case or capability it serves (`serves: seq/U7`).
- Every CLI subcommand and flag, and every LSP capability, appears in some use case.
- Every journey step has at least one use case.
- Two numbers for M1. *Baseline:* of the gaps known when the catalogue was
  first written, how many remain; this only goes down. *Readiness:* every
  known M1-critical gap, including ones found since; this can go up, and it
  is the release criterion. Finding a gap is good work.
- IDs are never reused.
- Every trial friction is matched to a use case; one with no match is an
  *escape* and is added. Escapes are a signal, not a proof the list is
  complete, so each catalogue is also reviewed once by an agent that hasn't
  seen it, and we try held-out problems against it.

These prove that the docs and the product are linked. Whether the list is
complete and the designs are good is what the review, comparisons and
trials are for.

### Statuses and milestones

Status: `supported`, `partial`, `gap`, `deferred` or `non-goal`.
Milestone, separately: `M1-critical`, `M1-differentiating`, `later`.

### Writing

Product docs follow `handbook/guidelines/writing-product.md`: start from a
person in a moment, show the interaction step by step, and name where it
hurts. A surface doc is under a page and holds current intent only; history
goes to RFCs.

## Rollout

1. Pilot on three surfaces that stress the process differently: std
   sequences, `cpl stress`, and editor completion. Each owner writes the
   surface doc, its use cases and acceptance cases. Another lane reviews it
   for missing needs and bad designs.
2. Judge the pilot: did it change a design before it was built, find needs
   the owner missed, expose a mismatch between surfaces, and produce useful
   tests? Can a later agent decide well from the docs alone?
3. Then roll out to every area, with the checks turned on.

## Cost

A first pass per pilot surface takes about an hour. Deep design takes as
long as it pays off, because designing is the work, not process. After
that, a change costs one use-case edit. The 10% rule applies to the
bookkeeping: IDs, statuses and links. If that costs more, we simplify it.

## Alternatives

- **Keep fixing frictions as they come.** It only finds problems after users hit them.
- **A requirements doc per feature.** It covers the feature but never the
  whole surface, so gaps between features stay invisible.

## Prior art

- **Rust RFCs:** a guide-level explanation comes before the reference.
- **Amazon's "working backwards":** write the user-facing announcement first.
- **Use-case catalogues and requirements traceability** in safety-critical
  engineering: every requirement has an ID and a test.
