# cco23p1

Score: 4/5
Dream: 4/5

Cold review against the friction's `## Dream`. Each residue class steps the windows (`Window.drop(r).step(k)`, line 21), its offsets are a prefix sum (line 22), the span classifies the chain (lines 25-27) and `Comb<Mod<1_000_003>>` counts (line 30): the header's argument.

## Checklist failures

- R1, minor: lines 7-8 read "Offsets spanning more than one value: no string; spanning two: forced", which contradicts itself; the code (line 25) means a span over 1 is impossible, span 1 is forced, span 0 is free.
- R3: `(0..=steps.len).map(i => off.prefix(i))` and two folds (lines 23-24) stand in for iterating the prefixes; reported.

## Owed rewrites

- Fix the header wording of lines 7-8 to "width > 1: none; width 1: forced; width 0: free".

## Language gaps

- `Prefix` is not iterable and has no `minmax` (lines 22-24): in the friction.
- `adjacent` on a table does not feed pairs to `map` (E0302, quoted): in the friction; the workaround is line 21.

## Dream notes

- One line replacing lines 21-24, judged as the port with that change: `total.adjacent.map(...).drop(r).step(k).prefixes.minmax()!` names the chain span directly. `prefixes` must include the leading 0 (port line 23); say so in the friction.
