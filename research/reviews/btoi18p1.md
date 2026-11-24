# btoi18p1 review

Score: 2/5
Dream: 2/5

Cold review of `port.cp` against the friction's `## Dream`, without the C++. The header gives
the matching argument: keep mutual pairs, match greedily from the leaves inward, then along each
remaining cycle, starting after a matched person if there is one. The port does this, but it
rebuilds the functional graph's structure by hand.

## Checklist failures
- idiomatic-code/R3: the ring walk (lines 46-50, `while loves[ring.last!] != start:
  ring.push(...)` and `fans[p] = 0`) re-derives std `Functional(loves).cycles()`
  (std/graph.cp), which lists every cycle in walk order. The friction says "std has no split of
  a functional graph into ... its cycles", which is wrong for the cycles half.
- The in-degree peel (lines 36-43) is hand-written as well. It interleaves with the matching,
  so it may have to stay; std `peel(g, on_remove:)` is the candidate to check.

## Owed rewrites
- Get the cycles from `Functional(loves).cycles()`. Keep the peel if `peel` can't carry the
  matching, and correct the friction to say only that.

## Language gaps
- The friction's "no functional-graph split" holds for the peel order at most: `Functional`
  has `cycles()`.

## Dream notes
The dream is one line, `let (rings, peel) = functional_graph(loves)`. It names the missing
object but not the matching over the peel and the rings, so it is not a program to judge the
port against. Write the whole thing (send back).
