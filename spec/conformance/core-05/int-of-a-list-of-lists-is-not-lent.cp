// expect: ok, output "1 3"
// rule: 05.2.1 05.3.5
// An Int read through two index levels is a copied value: it holds no lend, so it can be pushed.
var xs: List<List<Int>> = [[1]]
var ys: List<Int> = []
let x = xs[0][0]
ys.push(x)
let adj = [0..3] v => [w for w in 0..3 if w != v]
var out = List<Int>()
for u in 0..3:
    let v = adj[u][0]
    out.push(v)
print(ys.len, out.len)
