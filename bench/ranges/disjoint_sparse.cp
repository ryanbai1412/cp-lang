use std.sparse.{DisjointSparseTable}

let n = 100_000
domain I = 0..n
let a = [I] i => (i.to_idx * 37 + 13) % 1000
let dst = DisjointSparseTable<I, Int>(+, a)
var s = 0
var x = 9876543
for _ in 0..500_000:
    x = (x * 1103515245 + 12345) & 2147483647
    let l = x % n
    x = (x * 1103515245 + 12345) & 2147483647
    let r = x % n
    s += dst.fold(I(min(l, r))..=I(max(l, r)))
print(s)
