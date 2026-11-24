# noip20p1

Score: 3/5
Dream: none (no dream.cp; the ideal is in the friction report)

Scored cold against the checklist and the port. With no dream.cp the port scores at most 3 (README); against the checklist alone it reads as a 4.

Cold review: flow is exact in I128 units of 60^11, the most splits a path can see, so there is no fraction type and no gcd per edge; topo() orders the junctions. The final reduction divides out 2, 3 and 5, the only primes in the unit.

## Checklist failures

- Flow is kept exact in `I128` units of `60 ** 11`, a unit picked by hand from the most splits a path can see. A rational type would make the unit unnecessary.
- The printed fraction is reduced by a loop over `[I128(2), I128(3), I128(5)]` because the unit's only prime factors are 2, 3 and 5, which couples the output to the unit choice.
- `var holds = [Junction] u => if u.label <= m: unit else: I128(0)` marks the sources by a label comparison instead of a `Source = 1..=m` sub-domain.

## Gaps that need the language

- Exact fractions need a hand-picked common unit: research/frictions/noip20p1.md.
