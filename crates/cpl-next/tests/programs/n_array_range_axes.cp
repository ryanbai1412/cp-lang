// expect: ok, output "5 6\n7 0 24\n3 2"
// rule: R04.2.10 R07.2.17
// Each range axis of `[0..a, 0..b]T` is a domain of its own, so a tuple of `Int`s indexes the
// array part by part, each part checked against its own axis.
var g: [0..2, 0..3]Int = 0
let j = 2
g[1, j] = 5
g[0, 0] += 6
print(g[1, 2], g[0, 0])
fn grid(n: Int, m: Int) -> Int:
    var h: [0..n, 0..=m, 2]Int = 0
    h[n - 1, m, 1] = 7
    let (r, c) = (n - 1, m)
    h[r, c, 0] += h[r, c, 1]
    var total = 0
    for p in 0..n: total += 1
    h[r, c, 0] + total - n
print(grid(3, 2), g[1, 0], 2 * 3 * 4)
var w: [0..2, 0..2, 0..3]Str = ""
w[1, 0, 2] += "ab"
w[1, 0, 2] += "c"
print(w[1, 0, 2].len, w[1, 1, j].len + 2)
