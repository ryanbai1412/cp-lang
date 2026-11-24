# ioi17p4 review

Score: 3/5
Dream: 3/5

Scored cold against the checklist, the port and the dream (the `## Dream` section of research/frictions/ioi17p4.md).

The divide and conquer on cuts with the non-lollipop counts at both ends follows the header,
and the first 450 probes find `widest`. Half the file is the judge, reported. `ask` caches by
a flag table and updates `widest` as a side effect, reported as the missing `memo`/`also`.

## Checklist failures

- `(Box.first!..=Box(min(n, 450) - 1)).find(i => ask(i).l + ask(i).r == 0)` (line 50) asks
  twice per box and relies on `ask` raising `widest` inside a `find` predicate (R7); the
  reported `memo` covers the caching, not the hidden effect. Asking the 450 boxes in a loop
  and then testing for the diamond would show it.
- `Box.first!..=Box(min(n, 450) - 1)` builds a prefix from `Int`; reported.

## Language gaps

- `@interactor`, `memo`/`also`, `Box.take(k)`: in research/frictions/ioi17p4.md.

## Dream notes

- `fn search(span, before, after) -> ?Box` has no body, so the recursion, the part that needs
  care, is not shown; it keeps the same effect-in-`find` probe. It should show `search` over
  `Range<Box>`.
