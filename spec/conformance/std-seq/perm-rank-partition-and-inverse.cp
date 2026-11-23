// expect: ok, output "2 1 0 2\n1 3\n1 3 3 3 0 0\n2 none 0\nYES"
use std.seq.{perm_rank, perm_unrank, partial_inverse}
domain T3 = 0..3
domain T4 = 0..4
let p: [T3]T3 = [T3(1), T3(0), T3(2)]
print(perm_rank(p), perm_unrank<T3>(2))
for r in 0..24:
    let q = perm_unrank<T4>(r)
    assert perm_rank(q) == r
let a: [T3]Int = [1, 3, 5]
print(partition_point(a, x => x > 2).ord, partition_point(a, x => x > 9).ord)
let b: [T4]Int = [1, 3, 3, 5]
let (e3, e4, e0) = (b.equal_range(3), b.equal_range(4), b.equal_range(0))
print(e3.l, e3.r, e4.l, e4.r, e0.l, e0.r)
let c: [T3]?T3 = [Some(T3(2)), None, Some(T3(0))]
print(c.partial_inverse())
print("YES")
