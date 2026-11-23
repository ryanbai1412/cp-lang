// Source: DMPG '16 G2 (Kabane Apocalypse), https://dmoj.ca/problem/dmpg16g2
// Problem: an N x N grid (N <= 1000) of cells, each '.' or one of the letters A..E. Count the
//   axis-aligned sub-grids that contain all five letters.
// Solution: inclusion-exclusion over the 32 sets S of letters left out: add (-1)^|S| times the
//   sub-grids avoiding S. Fix the bottom row; a column is free for S while none of S's letters
//   occurs in it between the top and bottom rows. Raising the top from row 0 frees columns as
//   the last occurrence of a letter drops out; free runs merge in a Dsu per S, which keeps the
//   count of column spans inside free runs. O(32 N^2) Dsu steps.
// original: corpus/submissions/4653887.cpp

const M = 5
const MASKS = 32

let n: 1..=500 = read()
domain Row = 0..n
domain Col = 0..n
let g: [Row, Col]Char = read()

fn c2(x: Int) -> Int:
    x * (x + 1) / 2

var pop: [0..MASKS]Int = 0
var has_bit: [0..MASKS, 0..M]Bool = false
for mask in 0..MASKS:
    var tmp = mask
    for b in 0..M:
        if tmp % 2 == 1:
            pop[mask] += 1
            has_bit[mask, b] = true
        tmp = tmp / 2

var last: [0..n, 0..M]Int = -1
var dsu_p: [0..MASKS, 0..n]Int = -1
var blocked: [0..MASKS, 0..n]Int = 0
var total: [0..MASKS]Int = 0

fn dsu_find(mask: Int, u: Int) -> Int:
    var r = u
    while dsu_p[mask, r] >= 0:
        r = dsu_p[mask, r]
    var cur = u
    while dsu_p[mask, cur] >= 0:
        let nxt = dsu_p[mask, cur]
        dsu_p[mask, cur] = r
        cur = nxt
    r

fn dsu_size(mask: Int, u: Int) -> Int:
    -dsu_p[mask, dsu_find(mask, u)]

fn dsu_merge(mask: Int, u: Int, v: Int):
    var ru = dsu_find(mask, u)
    var rv = dsu_find(mask, v)
    if ru == rv:
        return
    if dsu_p[mask, ru] < dsu_p[mask, rv]:
        swap(mut ru, mut rv)
    dsu_p[mask, rv] += dsu_p[mask, ru]
    dsu_p[mask, ru] = rv

var ans = 0

for bot in 0..n:
    for c in 0..n:
        let ch = g[Row(bot), Col(c)]
        if ch != '.':
            last[c, Int(ch) - 65] = bot

    for mask in 0..MASKS:
        total[mask] = 0
        var run = 0
        for c in 0..n:
            dsu_p[mask, c] = -1
            var cnt = 0
            for b in 0..M:
                if has_bit[mask, b] and last[c, b] >= 0:
                    cnt += 1
            blocked[mask, c] = cnt
            if cnt == 0:
                run += 1
                if c > 0 and blocked[mask, c - 1] == 0:
                    dsu_merge(mask, c - 1, c)
            else:
                total[mask] += c2(run)
                run = 0
        total[mask] += c2(run)

    var ev_col: [0..n]List<Int> = []
    var ev_ch: [0..n]List<Int> = []
    for c in 0..n:
        for b in 0..M:
            let r = last[c, b]
            if r >= 0:
                ev_col[r].push(c)
                ev_ch[r].push(b)

    for top in 0..=bot:
        for mask in 0..MASKS:
            if pop[mask] % 2 == 1:
                ans -= total[mask]
            else:
                ans += total[mask]
        let ec = ev_col[top].len
        for ei in 0..ec:
            let c = ev_col[top][ei]
            let b = ev_ch[top][ei]
            for mask in 0..MASKS:
                if has_bit[mask, b]:
                    blocked[mask, c] -= 1
                    if blocked[mask, c] == 0:
                        if c > 0 and blocked[mask, c - 1] == 0:
                            total[mask] -= c2(dsu_size(mask, c - 1))
                            dsu_merge(mask, c - 1, c)
                        if c + 1 < n and blocked[mask, c + 1] == 0:
                            total[mask] -= c2(dsu_size(mask, c + 1))
                            dsu_merge(mask, c, c + 1)
                        total[mask] += c2(dsu_size(mask, c))

print(ans)
