# noip09p3

Score: 4/5
Dream: 4/5

Cold review: condense the SCCs, then a forward pass for the cheapest price from city 1 and a backward pass for the dearest to city n (lines 22-45), exactly the header.

## Checklist failures

- R3: the roads are read one at a time into `road` (lines 15-19); a typed read `[0..m](u: City, v: City, ways: 1..=2)` then the pushes would keep the read in one place.

## Owed rewrites

- Read the roads as one typed table.

## Language gaps

- The two DAG passes by hand: in research/frictions/noip09p3.md, now with the `cpl run` error (`dag.fold_from` resolves to the sequence fold).

## Dream notes

- None.
