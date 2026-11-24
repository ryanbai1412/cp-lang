# sdoi17r1p1

Score: 3/5
Dream: 4/5

Cold review: Mobius over gcds, g by a harmonic loop, prefix products and quotient blocks (lines 14-45), the header's derivation, but the exponent is picked by two `if`s that `Mod ** -1` already removes.

## Checklist failures

- R3: `if mu[k / d] == 1: g[k] *= up` / `if mu[k / d] == -1: g[k] *= down` (lines 28-31) where `g[k] *= fib[d] ** mu[k / d]` runs today.

## Owed rewrites

- `g[k] *= fib[d] ** mu[k / d]` (dream line 7).

## Language gaps

- The quotient-block `while` (lines 40-44): in research/frictions/sdoi17r1p1.md, now with the `cpl run` error for `quotient_blocks`. Its "`Mod ** -1` is not the inverse" is corrected there.

## Dream notes

- None.
