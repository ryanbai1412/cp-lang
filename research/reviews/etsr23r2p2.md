# etsr23r2p2

Score: 2/5
Dream: 3/5

Cold review of the last `port.cp` (the package was deleted in e0502f639 because its judge limits can't be found) against the friction's `## Dream`. The product interval DP then the prefix sum DP over saturated values, with `Pos`, `Pos.cuts` and `Val` domains, was the header's argument.

## Checklist failures
- idiomatic-code/R12: the port saturated at `CAP = 254`, the largest target, so an expression worth more than 254 counted as 254: `s = 300, k = 254` printed `0` (correct: `-1`), unreported. `CAP = 255` with `k` read as `0..=254` prints `-1` and passed its 10 cases; the friction now records it and its dream takes the bound.

## Owed rewrites
- If the package returns, take `CAP = 255 > k`.

## Language gaps
- No saturating integer: `type V = Sat<0..=254>` is `error[E0204]: no type or domain named Sat`; repro now in the friction.
- No `?T` relax: `x.lower(3)` is `error[E0203]: a ?Int has no method lower`; repro now in the friction.

## Dream notes
The dream names both DPs with `Sat` arithmetic, `@memo(array)` and `.lower`, but was written with the same wrong bound (`Sat<0..=254>`), now corrected.
