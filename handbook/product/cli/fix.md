# Fix

> 15 minutes in, `cpl check` shows four errors: a missing `use`, an
> `Int` where the function takes `?Int`, an old spelling the docs renamed,
> and an index over the wrong domain. Three of them name their fix. You
> don't want to type three edits the tool already knows.

You run `cpl fix a.cp`. It applies each fix an error carries, the same
quick fixes the editor offers, and re-checks; when no error is left it
applies the safe lint fixes too. It rewrites the file and prints what is
left: here the domain error, which names two candidates and so has no
single fix. The exit status is 1 while an error is left. The program prints
what it printed before; a fix never changes behaviour and never adds an
error.

**Off the happy path.** Two fixes touch the same text: the first applies,
the second waits for the next round. A fix would only be right with a
choice the tool can't make: it is offered in the editor, not applied. A
library renamed a function: its codemod (`cpl migrate`) is separate,
because a rename crosses files. A file that doesn't parse: only parser
fixes apply.

**Ideal against today.** The ideal is that every error with one right
answer is fixed by one command, in the CLI and the editor alike, and that
the user trusts it unread. Today the fix of every error and lint is tested
for adding no error, and `cpl fix` over every conformance case keeps
its output, every emitted error code has a trigger and a clean fix where
it names one, and the editor's preferred quick fix is the one `cpl fix`
applies. Gap: no test pins that a fix needing a choice is left unapplied.

**Against the usual setup.** clang-tidy `-fix` and `cargo fix` apply
compiler suggestions marked machine-applicable; CP Lang does the same with
one marker (a fix is applied only if it is safe), and adds a behaviour
check over the whole conformance suite.

**Dimensions.** Source of the fix (checker error, parser, lint, library
lint), safety (safe, needs a choice), overlap (alone, overlapping), file
state (parses, doesn't), where (CLI, editor, JSON), and what proves it
(re-check, same output).
