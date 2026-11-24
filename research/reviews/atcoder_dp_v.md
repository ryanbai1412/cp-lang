# atcoder_dp_v

Score: 4/5
Dream: 4/5

Cold review against the idiomatic checklist.

- R1: `limits:` and `original:` are missing.
- R2: one `tree.reroot` call is the editorial's product-of-(1 + child) rerooting, with
  no division (the Solution says why).
- R3: std `reroot` replaces the 673-token prefix/suffix C++.
- R5: `n`, `m` bounded; the residues are plain `Int` (friction 1).
- R10: 0.95x.

## Checklist failures

R1 (header), R5 (residue not a type, reported).

## Owed rewrites

Move to a runtime-modulus type when std has one (Content).

## Language gaps

No value type for a modulus read at run time (friction 1).
