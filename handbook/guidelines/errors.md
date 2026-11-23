# Writing errors, warnings and traps

| | |
|---|---|
| Kind | Guidelines |
| Owner | diagnostics |
| Describes | `crates/cpl-diag/`, `spec/diagnostics.md`, error sites in `crates/cpl-syntax/`, `crates/cpl-check/`, `crates/cpl-ir/`, trap messages in `crates/cpl-interp/`, `crates/cpl-cli/tests/diagnostics/` |

Read this before you add an error, change a message or add a fix. It applies
to every error (`E`), warning (`W`), lint (`L`) and runtime trap (`R`).

## A good error, by example

```
error[E0107]: `&&` is `and` in CP Lang
  --> main.cp:3:6
  |
3 | if a && b:
  |      ^^
  = help: replace with `and`
```

It points at the two characters to change, says the rule in a few words,
and carries a fix that `cpl fix` and the editor apply.

A bad one, for `rows.sort(by: (a, b) => (-b, a))` where `rows` holds
records:

```
error[E0309]: `-` can't be applied to a `(name: Str, score: Int)`
```

The real mistake is that `by:` takes a key of one element, not a
comparator of two. The error blames `-`, which the user wrote correctly for
what they meant, and gives no fix. It should point at `(a, b)`, say that
`by:` takes one element and returns its key, and show a key to start
from, such as `r => (-r.score, r.name)`.

## The ideas behind every rule

An error answers four questions, in this order: where, what rule, what to
do, and is that the whole story.

**Where.** The caret goes on the token the user must change, in the user's
own file. The checker often notices a mistake later than where it was made:
in the `sort` example it first trips over `-b`, but the mistake is `(a, b)`.
Report where the user will edit, not where the compiler noticed. A library
is never the place to edit, so an error that surfaces inside `std/` points at
the user's call and mentions the library line only as a note.

**What rule.** Say what the user wrote and the rule it breaks, in the words
the docs use. The user wrote `&&`, so the message says `&&`, not "unexpected
token". If the docs call it a "domain", the error does too.

**What to do.** Most mistakes have one right edit, and then the error carries
it, so `cpl fix` or the editor applies it in one keystroke. A fix is applied
without being read, so it must leave a program that checks and still does
what it did. When the right edit depends on what the user meant, as when
turning a comparator into a sort key, the error explains the shape of the
answer instead of guessing.

**The whole story.** One mistake gets one error. Ten echoes of the first
mistake hide the second. And the same file gets the same error in the
terminal, the editor and the browser playground, on every OS, because they
all call one front end.

Warnings and runtime traps follow the same ideas. A warning must be true: if
the analysis can't show the problem, it stays silent. A trap happens on input
the user can't see, so it shows the values that broke the rule, the call
stack and the test case, and says how to avoid it.

## Deciding a case nobody wrote down

Imagine a strong contestant at 3am, reading the error once. Do they know
which line to edit, why it is wrong, and what to type? If any answer is no,
the error is not done. When two messages are possible, choose the one that
mentions what the user wrote. When unsure whether a fix is always right,
leave it as help.

Every error a real user found unhelpful becomes a test case, so the suites
learn from what people actually write, not only from what we imagined.

## Where the rules are

The precise rules, each with an id like `errors/R3`, a reason, an example and
the test that enforces it, are in `handbook/guidelines/errors-checklist.md`,
together with what a new code needs and a checklist for landing an error.
The diagnostics charter, `handbook/domains/diagnostics.md`, lists every
quality an error must have and the suite that measures it.

## Changes

- 2026-11-23: rewritten as prose; the rules, codes and landing checklist moved to `handbook/guidelines/errors-checklist.md`.
- 2026-11-23: first version, from the diagnostics judgment rubric and
  catalogue rules.
