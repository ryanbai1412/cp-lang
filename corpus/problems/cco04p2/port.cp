// Source: CCO 2004 P2, Hockey Scores, https://dmoj.ca/problem/cco04p2
// limits: 1000 ms, 32 MB
// Problem: multitest; each test has s <= 1000 scores `x-y`, each possibly written in either
//   order, taken from games in progress or finished. Print the fewest games such that every
//   score occurs in one, then each game's scores on a line (a valid scenario, any one).
// Solution: write each score as (min, max); scores fit one game iff they form a chain with both
//   numbers non-decreasing. Take scores largest first and put each on the row whose front has
//   the smallest second number that is still at least its own, by binary search over the
//   fronts, or open a new row; rows are printed reversed. O(s log s).
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
