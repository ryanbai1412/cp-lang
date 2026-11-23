// expect: ok, output "none none 1 1 none none\nnone none 0 2 none none\n0\n0 1 2\n2\n3 0 none none\n0 2 4\n-1 6\nnone 0 none 2 3 1 2 none 4 none"
use std.seq.{prev_smaller, next_smaller, prev_same, next_same, spans, record_highs, perm_sign, perm_order}
domain T3 = 0..3
domain T4 = 0..4
domain T5 = 0..5
let a: [T3]Int = [2, 1, 3]
print(a.prev_smaller(), a.next_smaller())
let b: [T3]Int = [1, 2, 1]
print(b.prev_same(), b.next_same())
let c: [T3]Int = [2, 1, 1]
print(c.spans())
let d: [T4]Int = [1, 3, 3, 5]
print(d.first_gt(3), d.last_lt(3), d.first_gt(5), d.last_lt(1))
let e: [T5]Int = [3, 1, 4, 4, 5]
print(e.record_highs().collect())
let p: [T3]T3 = [T3(1), T3(0), T3(2)]
let q: [T5]T5 = [T5(1), T5(0), T5(3), T5(4), T5(2)]
print(perm_sign(p), perm_order(q))
let f: [T5]Int = [2, 2, 1, 2, 2]
print(f.prev_smaller(strict: false), f.next_smaller(strict: false))
