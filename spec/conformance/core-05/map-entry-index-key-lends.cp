// expect: ok
// rule: 05.3.5
// tool: explain
// lacks: "[copy]"
// `let xs = at[p.cut(r)]` reads an element of a `let` map, so it is lent whatever the key expression; a copy per query made round 29's 1968F take 33 s.
let n: 1..=10 = read()
let a = read<[n]Int>()
let p = a.scan(^)
let at = (0..=n).group(by: k => p.cut(k))
var s = 0
for r in 0..=n:
    let xs = at[p.cut(r)]
    s += xs[xs.bsearch(k => k > 0)]
print(s)
