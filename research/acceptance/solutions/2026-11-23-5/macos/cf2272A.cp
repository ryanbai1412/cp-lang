// A. Maximize One MEX
// https://codeforces.com/contest/2272/problem/A
// judge: generic
// limits: 1000 ms, 256 MB

@!multitest
let (n, k): (2..=100, 1..=n) = read()
let a: [0..n]0..n = read()

fn score(b: List<Int>) -> Int:
    var seen = [false for _ in 0..n]
    var m = 0
    var f: List<Int> = []
    for x in b:
        seen[x] = true
        while m < n and seen[m]: m += 1
        f.push(m)
    return f.sorted(desc: true)[k - 1]

var b = [x for x in a]
var best = score(b)
var pick = (-1, -1)
for i in 0..n:
    for j in i + 1..n:
        (b[i], b[j]) = (b[j], b[i])
        let s = score(b)
        if s > best:
            best = s
            pick = (i, j)
        (b[i], b[j]) = (b[j], b[i])
if pick.0 < 0: print(-1)
else: print(pick.0 + 1, pick.1 + 1)

@check
fn judge():
    let x: Int = read_output()
    var c = [v for v in a]
    if x != -1:
        let y: Int = read_output()
        (c[x - 1], c[y - 1]) = (c[y - 1], c[x - 1])
    var d = [v for v in a]
    var top = score(d)
    for i in 0..n:
        for j in i + 1..n:
            (d[i], d[j]) = (d[j], d[i])
            top max= score(d)
            (d[i], d[j]) = (d[j], d[i])
    assert score(c) == top
