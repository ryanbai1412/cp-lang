# ioi13p4 review

Score: 3/5
Dream: 4/5

Scored cold against the checklist, the port and the dream (the `## Dream` section of
research/frictions/ioi13p4.md).

The plan reads as the header's: settle doors in order, one query for the opening position,
then `first_true` over how many free switches to flip. But the search predicate `opens`
rewrites the captured trial `s` and asks the judge (lines 32-34), so the binary search runs on
a predicate with side effects, and the friction report's evidence is stale.

## Checklist failures

- `opens(k, to)` assigns `s[w]` for every free switch and calls `try_combination` inside the
  predicate passed to `first_true` (lines 32-37), and the loop then relies on `s` holding the
  last probe (R7). The query itself is an effect, but the trial can be built locally:
  `let trial = [Switch] w => ...` passed to `try_combination`, with `s` changed only by the
  settle lines 37-38. Not reported.
- `[Switch]0..=1` and `1 - to` for switch positions; reported.

## Owed rewrites

- Build each trial inside `opens` as a fresh array, as the dream's `opens` does, and keep the
  settled positions in their own `open: [Switch]0..=1`.
- research/frictions/ioi13p4.md describes the old port ("port lines 15-25: the hidden state is
  read from input and `try_combination` is a local fn with its own `tries` counter"); the port
  now talks to `interactor.py` through `ask("try", s)` and prints `answer`. The protocol gap
  stands; its Where line needs the current lines.

## Language gaps

- A function-call grader protocol and enum positions with an io encoding: in
  research/frictions/ioi13p4.md, with E0107/E0201 quoted.

## Dream notes

- Near the bar: `protocol Cave` with its limit, an `@interactor` judge over `Perm<Switch, Door>`
  and `enum Pos`. `first_where` is a second spelling of the port's `first_true`.
