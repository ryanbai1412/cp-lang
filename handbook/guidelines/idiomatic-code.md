# Writing idiomatic CP Lang

| | |
|---|---|
| Kind | Guidelines |
| Owner | idioms |
| Describes | `corpus/problems/*/port.cp`, `dream.cp`, `spec.cp`, `skills/cp-lang/`, `spec/idioms/`, examples in docs and on the site |

Read this before you write a port, a dream, a skill page or a docs example.
People and models read every program we publish as the way CP Lang is meant
to be written, so each one has to be the version we would teach. The
exhaustive rules, each with an id that reviews and lints cite, are in
`handbook/guidelines/idiomatic-code-checklist.md`.

## What good looks like

A loop translated from the C++:

```cp
var best = -1
var at = -1
for i in 0..n:
    if a[i] > best:
        best = a[i]
        at = i
print(at + 1)
```

The idiomatic version:

```cp
domain I = 1..=n
let a: [I]1..=1e9 = read()
print(I.max(by: i => a[i]))
```

The statement's bound is a type and the index space is a domain, so there
is no `+ 1` to forget. The search is a named library call, with no `-1`
standing in for "none".

## The ideas behind the rules

**Say the idea, not the bookkeeping.** A reader should see the editorial's
invariant, recurrence or algorithm. The header says the idea before the
code does. When std or the language has the structure the editorial names,
use it: a hand-rolled copy is longer, can be wrong, and hides what std is
missing.

**Write each fact once, as a type where you can.** The statement's bounds
become types, index spaces become domains, and "absent" becomes `?T`. Then
the compiler checks the input, picks the width and keeps vertices apart
from edges, and a change of mind is a one-line edit.

**Nothing surprising.** An expression doesn't hide a side effect, a result
isn't silently dropped, and the program uses the one spelling the spec
gives each idea, so whoever learns from it learns the current language.

**Clean and fast, or say why not.** Published code checks and lints clean,
and runs about as fast as the matched C++. When it can't, the gap is a
friction report with a run that shows it, sent to the domain that can fix
it.

## Deciding a case nobody wrote down

Ask what a contestant reading the editorial would expect to see, and what a
model copying this program would learn. If the honest answer is "a
workaround", write the workaround and report the gap; don't polish it into
something that looks intended.

## Changes

- 2026-11-23: first version, from the taste checklist in `handbook/process/porting.md` and `research/reviews/README.md`.
- 2026-11-23: rules moved to `idiomatic-code-checklist.md`; this document keeps the ideas.
