# sac22cc2p5

Score: 4/5
Dream: 4/5

Cold review: one Fenwick per parity with zeros at the other parity, a set and a range fold per query (lines 13-30), exactly the header.

## Checklist failures

- R3: queries are read by a hand `ty` branch (lines 22-30) where dream.cp reads them as a tagged `enum Q` (dream.cp line 10).

## Owed rewrites

- Read the queries as `read<[q]Q>()` and `match`, as dream.cp does.

## Language gaps

- A strided range fold: in research/frictions/sac22cc2p5.md.

## Dream notes

- None.
