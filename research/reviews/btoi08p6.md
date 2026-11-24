# btoi08p6

Score: 3/5

Cold review against the idiomatic checklist.

- R1: complete, including the checker line.
- R2: the box/suffix-max argument is the editorial, but the storage is the C++'s
  `l << 32 | r` packing (friction 1).
- R3: raw `1 << c.ord` subset masks where `Subset<C>` is the std spelling.
- R5: the packed value is an unbounded `Int`; the two sums each have a bound.
- R8: `if let ... else: true` instead of `is_none_or` (friction 2).
- R10: 0.47x time, 0.56x memory.

## Checklist failures

R2, R3, R5, R8.

## Owed rewrites

Store `(l, r)` records over `Subset<C>` (the probe shows they are 8 bytes); use
`is_none_or`.

## Language gaps

None; records of bounded fields already narrow.
