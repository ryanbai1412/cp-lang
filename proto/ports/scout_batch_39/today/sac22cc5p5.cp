// SAC '22 CC5 P5 (today/ runnable subset) — original: corpus/submissions/4575631.cpp
use std.strings.{Hash2D, HashVal}

let (n, m, k): (1..=5_000, 1..=5_000, 1..=5_000) = read()
domain Row = 0..n
domain Col = 0..m
let g: [Row, Col]Char = read()

let h = Hash2D<Row, Col>.new((r, c) => Int(g[r, c]))
var seen = Set<HashVal>()
for r in 0..=n - k:
    for c in 0..=m - k:
        seen.add(h.rect(Row(r)..=Row(r + k - 1), Col(c)..=Col(c + k - 1)))
print(seen.len)
