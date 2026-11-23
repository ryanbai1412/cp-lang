# Building debugging tools

| | |
|---|---|
| Kind | Guideline |
| Owner | debugger |
| Applies to | traces, `cpl stress`, `--explain`, `cpl replay`, `cpl dap`, the trace viewer |

Debugging tools are only useful if the contestant can believe them. Read
this before you change what a trace records, how stress finds or shrinks a
case, what `--explain` says, or what the debugger shows. Each rule is
spelled out, with its check, in `handbook/guidelines/debugging-checklist.md`.

## The picture

A contestant gets a wrong answer at 3am. They run `cpl stress a.cp
--explain` and in a second see a three-number input, the line where the
solution and the brute first disagree, and a trace they can step through
in their editor. Three things make that trustworthy.

**The tools never disturb the run.** A traced run prints what an untraced
one prints, and the same seed gives the same input on every machine and
with any `--jobs`. If watching the program could change it, nothing the
trace shows could be trusted.

**The tools speak the user's program.** The trace shows the file's own
lines and values as the source writes them: a position in `domain Cell =
(R, C)` as its parts, and a call into a library such as `s.set(B(2),
true)` as one step at the call, not steps inside the library. The same
holds for every library, std or the user's own; no std type gets a special
case in the tracer.

**Explanations come from evidence.** `--explain` names a line because both
recordings show a reason, such as a value that differs, an output line that
differs, or a state that repeats. It never matches a known program or bug.
The seeded bugs measure how often it is right, and that count never falls.

## Deciding a case nobody wrote down

Ask what the contestant would see in a C++ debugger they trust, and what
they would need instead to find the bug faster. Prefer a general mechanism
any library can use over a fix for one type, store what changed rather than
whole values, and add a traced construct to the VM rather than to the
tree-walker. When an output format must change, add a field that carries
information only when present, and change the schema and viewer tests with
it.

## Changes

- 2026-11-23: first version.
