# mccc5j5

Score: 3/5
Dream: 4/5

Cold review: the aligned-block argument is in the header and `most` reads as it, but the block maxima are a hand-built tower whose depth is a bare 20, and the friction still describes the earlier `SparseTable` version.

## Checklist failures

- R1/R4: `20` appears twice (lines 16 and 24) and the header does not derive it (2^20 > 1e6).
- R3: `tower` (lines 15-18) is a hand-built table of aligned-block maxima; the friction says the port folds a std `SparseTable` and clips with `min(.., n) - 1`, neither of which is in the port.

## Owed rewrites

- Name the depth once (`let levels = bit_len(n)`) and use it for both loops.

## Language gaps

- The hand tower replacing `SparseTable` is unreported (cc8e065b6 swapped it in with no measurement); filed in research/frictions/mccc5j5.md.

## Dream notes

- `leaf` and `block(m ^ k, d)` are undefined; the dream should define the block as the header does.
