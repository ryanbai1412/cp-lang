// expect: ok, output "2 10 YES YES\n20 4 YES YES\n120 5"
use std.sparse.{SparseTable, DisjointSparseTable}
domain D = 0..3
let a: [D]Int = [D] v => [4, 6, 10][Int(v)]
let s = SparseTable(gcd, a)
print(s.all, s.get(D(2)), s.try_fold(D(1)..D(1)) == None, s.try_fold(D(1)..) == Some(2))
let t = DisjointSparseTable((x, y) => x + y, a)
print(t.all, t.get(D(0)), t.try_fold(D(2)..D(2)) == None, t.try_fold(D(0)..D(2)) == Some(10))
domain E = 0..5
let b: [E]Int = [E] v => Int(v) + 1
print(DisjointSparseTable((x, y) => x * y, b).all, SparseTable(max, b).all)
