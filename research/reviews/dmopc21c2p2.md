# dmopc21c2p2

Score: 4/5
Dream: 4/5 (the friction's `## Dream`)

Scored cold against the checklist and the port.

A permutation and its inverse kept in step under value swaps and position swaps; a query prints the target composed with the inverse.

## Checklist failures

- `perm` and `at_pos` are two arrays kept inverse by `swap_positions` by hand; a permutation type with both directions would keep them in step.
- `[Pos] i => Val.nth(i.ord)` builds the identity permutation through ordinals.
- The query kind is a `Str` compared with `"B"` and `"E"`, with an `else` for the third kind.
