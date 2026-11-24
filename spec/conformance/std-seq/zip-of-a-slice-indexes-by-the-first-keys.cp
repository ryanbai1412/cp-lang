// expect: ok, output "3\n3 8\n2 7\n1 2 3"
// rule: 04b.1.3
// A zip keeps the first sequence's keys: `z[k]` reads the first at key `k` and the second at k's position.
use std.seq.{zip}
let a = [1, 2, 3, 4]
let z = zip(a[1..], [7, 8, 9, 10])
print(z.len)
print(z[2])
print(z.nth(0))
print(" ".join(["{k}" for k in z.keys]))
