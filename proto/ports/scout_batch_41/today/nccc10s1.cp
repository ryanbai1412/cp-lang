// Mock CCC '22 Contest 2 S1 (nccc10s1), buildable under proto/cpc today. See ../nccc10s1.cp.
// original: corpus/submissions/4214147.cpp
let n: 1..=1_000 = read()
let k: 1..=10 = read()
let s: [0..n, 0..k]Char = read()
var masks: [0..n]Int = 0
for i in 0..n:
    var m = 0
    for j in 0..k:
        m = m * 2 + (if s[i, j] == 'T': 1 else: 0)
    masks[i] = m

var lim = 1
for _ in 0..k: lim *= 2

fn popcnt(x: Int) -> Int:
    var y = x
    var c = 0
    while y > 0:
        c += y & 1
        y = y >> 1
    c

var ans = 0
for cand in 0..lim:
    var worst = k + 1
    for i in 0..n:
        worst min= popcnt(cand ^ masks[i])
    ans max= worst
print(ans)
