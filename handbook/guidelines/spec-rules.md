# Writing spec rules

| | |
|---|---|
| Kind | Guidelines |
| Owner | spec |
| Describes | `spec/core/`, `spec/lib/`, `spec/tools/`, `decisions.md`, `handbook/language/` |

Read this before you add or change a rule in `spec/`. A rule is a sentence
that ends in its id, `(R07.4.11)`, and says what a program means or why it
is rejected. The reasons behind each area are in `handbook/language/`. This
doc says how to write a rule that holds up.

## A good rule, by example

> `w.drop(k)`, `w.take(k)`, `w.drop_last(k)`, `w.take_last(k)` are the same
> view as `w`, clamped: `w.drop(k)` is `w[w.l.next(k) ?? w.r..]`. (R07.4.11)

One sentence covers four methods on every view, defines them through a
form the reader already knows, and says what happens past the end. A
special case written for one view would have needed three more rules.

## How to think about a rule

When a reader trips over something, look for the general rule that would
have answered them, not a patch for their one case, and remove the special
cases it replaces. Give each operation one spelling and let the argument's
type choose the behavior. Say for every checked operation what it means,
what the compiler can prove, and what each build checks. Keep the chapters
and `std` saying the same thing, and change the matching
`handbook/language/` doc together with the rule.

The full list, with ids that checks cite, is [spec-rules-checklist.md](spec-rules-checklist.md).

## Changes

- 2026-11-23: first version; rules moved to the checklist.
