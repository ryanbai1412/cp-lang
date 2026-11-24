// expect: ok, output "3\n3"
// rule: 10.1.1
// input: "2\n3\n3\n"
// A `mut` argument is lent for the call and kept by nothing, so `rng.shuffle(mut perm)` on a name above `@!multitest` stores no position.
use std.rand.{Rng}
var rng = Rng.new(48)
@!multitest
let n: 1..=5 = read()
domain P = 0..n
var perm: [P]Int = [P] p => p.ord + 1
rng.shuffle(mut perm)
print(perm.sum() - 3)
