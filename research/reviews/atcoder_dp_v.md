# atcoder_dp_v

Score: 4/5
Dream: 3/5

Cold review against the idiomatic checklist, after the `Mod(m)` rewrite.

- R1: `limits:` is missing.
- R2: one `tree.reroot((*), lift: x + 1)` call is the editorial's product of (1 + child)
  rerooting, with no division (the Solution says why).
- R3: std `reroot` replaces the 673-token prefix/suffix C++.
- R5: `type Z = Mod(m)` makes the residue a type; no hand `% m`.
- R10: 0.95x.
- Dream: still the hand-reduced `% m` form, so it scores below the port.

## Checklist failures

R1 (header).

## Owed rewrites

Header lines; bring dream.cp to the `Mod(m)` form.

## Language gaps

None; `Mod(m)` was a docs-discoverability gap (friction 1, Content).
