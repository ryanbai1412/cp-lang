// COCI 2006 Debug: in a grid of bits, find the largest square (side at
// least 2) that looks the same after half a turn. Print its side, or -1.
let rows: 1..=300 = read()
let cols: 1..=300 = read()
domain Row = 1..=rows
domain Col = 1..=cols
domain Bit = '0'..='1'
let g: [Row, Col]Bit = read()                                       // WISH G8: packed rows of a Char domain

// Half a turn sends a square of g to the mirrored square of the turned
// grid, so one hash comparison decides a square.
let seen = Hash2D(g)                                                // WISH G7: hash of any rectangle
let turned = Hash2D([Row, Col] (r, c) => g[r.mirror, c.mirror])

fn same_turned(rs: Range<Row>, cs: Range<Col>) -> Bool:
    seen.of(rs, cs) == turned.of(rs.mirror, cs.mirror)              // WISH G2: the mirror image of a range

let best = max(size for size in 2..=min(rows, cols)
               if any(same_turned(rs, cs) for rs in Row.windows(size) for cs in Col.windows(size)))   // WISH F6
print(best ?? -1)
