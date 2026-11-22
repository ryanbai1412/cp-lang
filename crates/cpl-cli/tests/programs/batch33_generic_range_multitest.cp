use std.sparse.{SparseTable}

fn best<D>(a: [D]Int) -> ?D:
    var at: ?D = none
    for i in D:
        if let b = at, a[i] <= a[b]: continue
        at = some(i)
    at

fn solve(k: Int) -> Int:
    let a: [0..k]Int = [0..k] i => (i * 7) % 5
    let p = prefix(a)
    let at = best(a) ?? -1
    p.fold(0..k) * 100 + at + p.prefix(k / 2)

@!multitest
let n: Int = read()
print(solve(n), best([1..=n] i => (i * 3) % 7) ?? -1)
let xs: [0..n]Int = read()
let st = SparseTable(min, xs)
print(st.fold(0..n), st.fold(1..2))
var f = Fenwick(xs)
f.add(0, 5)
print(f.fold(0..n), f.prefix(1))
