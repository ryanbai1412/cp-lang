// Source: DMPG '16 G2 (Kabane Apocalypse), https://dmoj.ca/problem/dmpg16g2
// limits: 1400 ms, 256 MB
// Problem: an N x N grid (N <= 1000) of cells, each '.' or one of the letters A..E. Count the
//   axis-aligned sub-grids that contain all five letters.
// Solution: inclusion-exclusion over the 32 sets S of letters left out: add (-1)^|S| times the
//   sub-grids avoiding S. Fix the bottom row; a column is free for S while none of S's letters
//   occurs in it between the top and bottom rows. Raising the top from row 0 frees columns as
//   the last occurrence of a letter drops out; free runs merge in a Dsu per S, which keeps the
//   count of column spans inside free runs. O(32 N^2) Dsu steps.
// original: corpus/submissions/4653887.cpp
// Bounds: n: 1..=1000

const M = 5
const MASKS = 32

let n: 1..=1000 = read()
domain Row = 0..n
domain Col = 0..n
let g: [Row, Col]Char = read()

fn c2(x: Int) -> Int:
    x * (x + 1) / 2

domain Mask = 0..MASKS
let left_out = [Mask, 0..M] (mask, b) => (mask.ord >> b) % 2 == 1
let sign = [Mask] mask => if popcount(mask.ord) % 2 == 1: -1 else: 1
let holding = [0..M] b => Mask.filter(mask => left_out[mask, b]).collect()

// last[c, b]: the lowest row so far (at or above bot) where letter b last occurs in column c.
var last: [Col, 0..M]-1..n = -1
// seen[c]: the letters occurring in column c at or above bot, as a bit mask.
var seen: [Col]0..MASKS = 0
var ans = 0

for bot in Row:
    for c in Col:
        let ch = g[bot, c]
        if ch != '.':
            last[c, Int(ch) - 65] = bot.ord
            seen[c] |= 1 << (Int(ch) - 65)

    // blocked[mask, c]: letters of mask still present in column c between top and bot.
    var blocked: [Mask, Col]0..=M = [Mask, Col] (mask, c) => popcount(mask.ord & seen[c])
    var runs = [Mask] _ => Dsu<Col>()
    var total = [Mask] _ => 0
    for mask in Mask:
        var run = 0
        for c in Col:
            if blocked[mask, c] == 0:
                run += 1
                if c.ord > 0 and blocked[mask, Col(c.ord - 1)] == 0:
                    runs[mask].unite(Col(c.ord - 1), c)
            else:
                total[mask] += c2(run)
                run = 0
        total[mask] += c2(run)

    var drops: [Row]List<(c: Col, b: Int)> = []
    for c in Col:
        for b in 0..M:
            if last[c, b] >= 0:
                drops[Row(last[c, b])].push((c, b))

    for top in Row(0)..=bot:
        for mask in Mask:
            ans += sign[mask] * total[mask]
        for (c, b) in drops[top]:
            for mask in holding[b]:
                blocked[mask, c] -= 1
                if blocked[mask, c] == 0:
                    for d in c.ord - 1 ..= c.ord + 1:
                        if d != c.ord and d in Col and blocked[mask, Col(d)] == 0:
                            total[mask] -= c2(runs[mask].size(Col(d)))
                            runs[mask].unite(c, Col(d))
                    total[mask] += c2(runs[mask].size(c))

print(ans)
