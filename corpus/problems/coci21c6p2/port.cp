// Zemljište, in the forms proto/cpc builds today. See ../coci21c6p2.cp.
// original: corpus/submissions/4529998.cpp
let n: Int = read()
let m: Int = read()
let a: Int = read()
let b: Int = read()
let grid: [n, m]Int = read()
let lo = min(a, b)

var best: ?Int = none
fn offer(s: Int): best min= abs(s - a) + abs(s - b)

for top in 0..n:
    var col: [m]Int = 0
    for bot in top..n:
        for c in 0..m: col[c] += grid[bot, c]
        var r = 0
        var s = 0
        for l in 0..m:
            while r < m and s < lo:
                s += col[r]
                r += 1
            offer(s)
            if r - l >= 2: offer(s - col[r - 1])
            s -= col[l]
print(best!)
