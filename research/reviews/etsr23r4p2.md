# etsr23r4p2

Score: 2/5
Dream: 4/5

Re-ported in v0.3 style.

## Cold review (HEAD)

Port 2/5, dream 4/5. The dream's header derives the settling argument, `pow_exp` over an `Exp` type carries the generalized Euler rule, and the segment tree is a `[Step]Z` sum under a saturating shift (dream lines 16-31). The port keeps the same tree (lines 63-75) but spells the Euler rule by hand.

## Checklist failures

- `power` and `reach_of` (lines 41-56) carry the `(big, val)` pair and the "did `base^k` reach m" loop by hand. That is the `Exp` gap, in the friction.
- Unreported: `low`/`high` (lines 27-38) are baby-step/giant-step power tables with `step = 1 << 15`, next to a std `pow_mod` the port already calls on line 36. Neither the header nor the friction says why the tables beat `pow_mod`; if speed is the reason it needs a measured performance friction (idiomatic-code/R10), otherwise `pow_mod` says it.
- The clamp `min(depth, moduli.len - 1)` is written twice (lines 23 and 54); `power` can call `modulus(depth)` (idiomatic-code/R4).
- The comment on lines 25-26 describes `power`, but sits above `step` and is followed by a second comment for the tables.

## Language gaps

- `Exp` for the generalized Euler theorem: in the friction.
- The friction's "saturating shift action is spelled by hand" entry is stale: line 67 already uses `s.shift_saturating(k)`. Its "chain's keys are not a domain" entry says the dream uses `iterate(...).take_while(...)`, but dream line 11 keeps the `while ... push(phi(...))` loop.

## Dream notes

- Dream line 13 keeps the `min(depth, chain.len - 1)` clamp; `depth.next ?? depth` over a `Step` domain (the friction's own ideal) would drop it.
