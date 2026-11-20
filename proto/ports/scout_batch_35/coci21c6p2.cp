// COCI 2021/22 #6 Zemljište: a grid of positive numbers and two targets a, b.
// Over all sub-rectangles with sum s, minimise |s - a| + |s - b|.
//
// The cost is flat between the targets and grows outside, so for a fixed
// band of rows and a fixed left column only two right ends matter: the first
// one whose sum reaches the lower target and the one before it. That first
// right end only moves right as the left column does.
// original: corpus/submissions/4529998.cpp
let (n, m): (1..=500, 1..=500) = read()
let (a, b): (1..=1e9, 1..=1e9) = read()
let grid: [0..n, 0..m]1..=1e9 = read()
let lo = min(a, b)

var best: ?Int = none
fn offer(s: Int): best min= abs(s - a) + abs(s - b)

for top in 0..n:
    var col: [0..m]Int = 0              // column sums of rows top..=bot
    for bot in top..n:
        for c in 0..m: col[c] += grid[bot, c]
        var (r, s) = (0, 0)             // s = sum of col[l..r]
        for l in 0..m:
            while r < m and s < lo:
                s += col[r]
                r += 1
            offer(s)
            if r - l >= 2: offer(s - col[r - 1])
            s -= col[l]
print(best!)
