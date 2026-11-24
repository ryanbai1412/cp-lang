# Friction report: noi04p1

## Dream

The port is the dream: one online pass over a `SortedBag` of base salaries.

## Offline Fenwick becomes an online `SortedBag`

The v0.2 port read the commands twice: once to collect every base salary into a `Set`, sort
it and map it to slots, then again over a `Fenwick` with a hand descent for the k-th. The
rewrite keeps the bases in std `SortedBag`. A cut is `drain(while: b => b + raise < floor)`
and "F k" is `nth(len - k)`. The commands are a `match`, and the input bounds are in the read
type.

## Verification

30 random cases agree with the v0.2 port and 30 with `orig.cpp`. The max case runs in 0.09 s.
Corpus tests pass.
