// COCI 2006 Debug, as it has to be written today: a 2-D hash built from
// prefix sums of weighted cells. Builds; agrees with 4183051.cpp on 500
// random grids fed by a script (the typed generator cannot make this input).
const P = 1000000007
let rows: 1..=300 = read()
let cols: 1..=300 = read()
domain Row = 1..=rows
domain Col = 1..=cols
let g: [Row, Col]Char = read()

var row_pow: [Row]Int = 1
var col_pow: [Col]Int = 1
for r in Row:
    if let up = r - 1: row_pow[r] = row_pow[up] * 131 % P
for c in Col:
    if let left = c - 1: col_pow[c] = col_pow[left] * 137 % P
fn weight(r: Row, c: Col, ch: Char) -> Int: (ch - '0' + 1) * row_pow[r] % P * col_pow[c] % P
let seen = prefix([Row, Col] (r, c) => weight(r, c, g[r, c]))
let turned = prefix([Row, Col] (r, c) => weight(r, c, g[r.mirror, c.mirror]))

fn killer(r: Row, c: Col, size: Int) -> Bool:
    if let r2 = r + (size - 1) and let c2 = c + (size - 1):
        let here = seen.sum(span(r, r2), span(c, c2)) % P
        let there = turned.sum(span(r2.mirror, r.mirror), span(c2.mirror, c.mirror)) % P
        return here * row_pow[r2.mirror] % P * col_pow[c2.mirror] % P == there * row_pow[r] % P * col_pow[c] % P
    false
let best = max(size for size in 2..=min(rows, cols) if any(killer(r, c, size) for r in Row for c in Col))
print(best ?? -1)
