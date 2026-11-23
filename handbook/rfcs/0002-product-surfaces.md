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
chosen design and the evidence that the design works. We write this once,
exhaustively, then keep it current. Gaps show up as listed, unbuilt use
cases rather than as user reports.

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
   `handbook/product/journeys.md`, refined into workflows, each with an
   owner and its own acceptance cases. Few and not exhaustive.
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

The language itself keeps `handbook/language/` and `spec/`; its use cases
are the conformance and corpus programs it already has.

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

### How we design a surface

1. **Understand the need:** the journeys it serves, how the 877 corpus ports
   use it, STL, Python, Rust and AtCoder Library equivalents, textbook topic
   lists, and trial reports.
2. **Dream:** write the ideal code or interaction for representative
   problems before looking at what CP Lang does today.
3. **Design:** choose the model. Consider the status quo and real
   alternatives seriously; don't invent a second option to tick a box.
4. **Validate:** try the design against every use case and many real
   programs, and look for counterexamples. New needs found here go back
   into the list.
5. **Commit:** the use cases, their designs and their acceptance cases.

Depth depends on how uncertain the decision is and how costly it is to
reverse:

| Level | Example | Needs |
|---|---|---|
| A | A bug, or a listed use case being built | Nothing extra |
| B | A new use case, or a changed design | Its use-case entry |
| C | A new surface, or a model across surfaces (indices) | The five steps, an independent review by another lane, and Ryan's review |

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
- Within the M1 cohort, the count of unsupported use cases only goes down.
  Use cases discovered later join the next cohort, so finding gaps is
  rewarded, not punished.
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
2. Judge the pilot: did it find needs we'd missed, improve a design, and
   produce useful tests, for how much time?
3. Then roll out to every area, with the checks turned on.

## Cost

About an hour per pilot surface, and one use-case edit per change after
that. We'll notice it costs too much if lanes spend more than 10% of their
time on these docs without the pilot finding real gaps.

## Alternatives

- **Keep fixing frictions as they come.** It only finds problems after users hit them.
- **A requirements doc per feature.** It covers the feature but never the
  whole surface, so gaps between features stay invisible.

## Prior art

- **Rust RFCs:** a guide-level explanation comes before the reference.
- **Amazon's "working backwards":** write the user-facing announcement first.
- **Use-case catalogues and requirements traceability** in safety-critical
  engineering: every requirement has an ID and a test.
