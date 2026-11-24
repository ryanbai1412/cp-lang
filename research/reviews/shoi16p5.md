# shoi16p5

Score: pending
Dream: pending

Cold review: positions are a 1-based Pos domain read as 0..=1e4 values; each is a std Affine map, the run counts are one table computed once, and the SegTree's fold nests each map inside the one before it. Updates read as (Pos, 0..=1e4) and need no index shift.
