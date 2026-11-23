# Performance

| | |
|---|---|
| Kind | Guidelines |
| Owner | perf |
| Describes | `crates/cpl-codegen/`, `bench/`, `cpl bench`, tuned variants (`perf.cp`); every rule is in `performance-checklist.md` |

Read this before you claim a program is fast, compare CP Lang with C++,
add an optimization to codegen, or write a tuned variant. The exhaustive
rules, each with an id that tests cite, are in
[performance-checklist.md](performance-checklist.md).

## What we are after

A contestant should never switch to C++ for speed. The program they would
naturally write in CP Lang should run about as fast as the C++ a strong
contestant writes for the same algorithm, and when it doesn't, they should
be able to see why and fix it without leaving the language. None of this
may change what the program does.

## Compare like with like

A performance number means something only if the two programs do the same
work. `corpus/problems/cf2084h/` holds the contestant's C++ (`orig.cpp`),
the natural port (`port.cp`) and a tuned port (`perf.cp`), all using the
same stack-of-runs algorithm. The harness builds each with the judge's
compiler and flags, runs them on the same maximum-size input, checks that
they print the same answer, and only then compares times: 1.23x for the
natural port, 0.95x tuned.

Timing a segment tree against a Fenwick tree, a `-O0` build, or a ten-line
input measures something else. A pair that can't be matched is listed with
its reason, never silently dropped, because dropped rows hide the worst
cases.

## How fast is fast enough

The natural program should be within 1.1x of matched C++ for a typical
problem and never worse than 1.5x. For every program that misses 1.1x, a
tuned version written in the language (narrower types, `unchecked:`, a
different layout) must reach it; if none can, that is a gap in the
language, written down with whoever can close it. Abstractions such as
domains, optionals, iterators and operation values should cost nothing at
all, and the emitted C++ should fit comfortably under the judges' 64 KB.

## Faster, never different

An optimization is only correct if no program can tell it happened. It
may not change what is printed, in what order, how many times a call with
effects runs, where a program traps, or how often a lazy view is
evaluated. So each rewrite rests on a fact the compiler has proven (this
call has no effects, this value is never changed while it is borrowed),
never on a guess. Both engines, the interpreter and the C++, must still
agree on every program, and the program that prompted the change becomes a
test.

## Read in place, call directly

Most hidden slowness in generated code comes from two things: copying a
list that is only read, and calling a function through an indirect
`std::function`. Codegen reads values in place unless the program needs
its own copy, and calls a known function directly. Two counts over the
whole corpus track what is left and only go down.

## No secrets, no special cases

Codegen may choose a representation on its own only when the choice is
never worse, and it never swaps the algorithm the program names. Each
choice it makes is shown by `cpl explain`. Each optimization is one
general rule, not a case for one library function, so it keeps working
when the library changes and helps user code too. Builds are deterministic,
and every published number says how to reproduce it.

## When the rules don't say

Ask whether a contestant reading the source could predict the cost and the
behaviour of the C++. If a change makes that harder, it needs a stronger
reason than a faster benchmark.
