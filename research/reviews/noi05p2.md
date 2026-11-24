# noi05p2

Score: 4/5
Dream: 4/5

Port 4/5, dream 4/5 (the friction's `## Dream`). A std `SeqFold` over `MaxSub` with an assignment action, mirror for reversal, and each command one call (lines 42-47): the editorial's splay tree with nothing hand-rolled.

## Checklist failures

None beyond the friction (explicit type arguments on `SeqFold`, the spelled-out assignment `Action`, the `x => x.reversed()` mirror).

## Owed rewrites

- `_ =>` stands for `GET-SUM` (line 47); naming the command, as the dream does, keeps the dispatch readable.

## Language gaps

All in the friction.

## Dream notes

Close to the port with the inference gaps closed; `act: assign` and `mirror: .reversed` assume std names and method references the friction reports as missing.
