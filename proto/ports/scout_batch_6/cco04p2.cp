// CCO 2004 P2, as its accepted solution reads it: each test has n tiles,
// written `a-b`, and a tile may be turned around. Put them in as few rows as
// possible so that along a row neither number of a tile is smaller than the
// same number of the tile before it. Print the number of rows, then the rows.
// original: corpus/submissions/5430096.cpp
@multitest
let n: 1..=1e5 = read()
let given: [n](a: Int, "-", b: Int) = read()                                        // WISH S8
let tiles = sorted((min(t.a, t.b), max(t.a, t.b)) for t in given)

// Largest tile first, each on the row whose front tile has the smallest b
// that is still large enough. The rows stay in order of that b, so the row is
// found by binary search, as in patience sorting.
var rows: List<List<(a: Int, b: Int)>> = []
for t in tiles.rev:
    let r = rows.partition_point(row => row.last!.b < t.b)                          // WISH S13 (D8), S6
    if r < rows.len: rows[r].push(t)
    else: rows.push([t])
print(rows.len)
for row in rows: print(" ".join("{t.a}-{t.b}" for t in row.rev))                    // WISH G4
