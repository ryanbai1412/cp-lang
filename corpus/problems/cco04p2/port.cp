// CCO 2004 P2, as its accepted solution reads it: each test has n tiles,
// written `a-b`, and a tile may be turned around. Put them in as few rows as
// possible so that along a row neither number of a tile is smaller than the
// same number of the tile before it. Print the number of rows, then the rows.
// original: corpus/submissions/5430096.cpp
type Tile = (a: Int, b: Int)

fn read_tile() -> Tile:
    let word: [domain W]Char = read()
    var ends: List<Int> = [0]
    for c in word:
        if c == '-': ends.push(0)
        else: ends[ends.len - 1] = ends[ends.len - 1] * 10 + (c - '0')
    (min(ends[0], ends[1]), max(ends[0], ends[1]))

@!multitest
let n: 1..=1e5 = read()
var tiles: List<Tile> = []
for _ in 0..n: tiles.push(read_tile())

// Largest tile first, each on the row whose front tile has the smallest b
// that is still large enough. The rows stay in order of that b, so the
// row is found by binary search, as in patience sorting.
var rows: List<List<Tile>> = []
sort(mut tiles)
for t in tiles.rev:
    let r = Int(rows.partition_point(row => row.last!.b >= t.b))
    if r < rows.len: rows[r].push(t)
    else: rows.push([t])
print(rows.len)
for row in rows:
    print(["{t.a}-{t.b}" for t in row.rev])
