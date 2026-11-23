// expect: ok, output "0 1 2 0 0 1 YES\nYES"
use std.dp.{monge_path, monge_paths}
let r = monge_paths([0, 1, 2], (a, b) => (b - a) * (b - a))
print(r.val, r.prev, yes(monge_path([0, 1, 2], (a, b) => (b - a) * (b - a)) == (val: 2, count: 2)))
var ok = true
var seed = 7
for t in 0..60:
    let n = 1 + t % 23
    var xs: List<Int> = []
    var x = 0
    for _ in 0..n:
        seed = (seed * 1103515245 + 12345) % 2147483648
        x += seed % 5
        xs.push(x)
    let c = (a: Int, b: Int) => (b - a) * (b - a) + 3
    let got = monge_paths(xs, c)
    var best = [0 for _ in 0..n]
    for i in 1..n:
        best[i] = best[0] + c(xs[0], xs[i])
        for j in 1..i:
            let v = best[j] + c(xs[j], xs[i])
            best[i] min= v
    if got.val != best: ok = false
    for i in 1..n:
        if got.val[got.prev[i]] + c(xs[got.prev[i]], xs[i]) != got.val[i]: ok = false
print(yes(ok))
