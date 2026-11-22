// expect: ok, output "6\n5"
// rule: 10.1.1
// input: "2\n3\n1 2 3\n1\n5\n"
use std.math.{Sieve}
let sv = Sieve(1e6)                   // runs once

@!multitest
let n: 1..=2e5 = read()               // runs once per test case
let a = read<[n]Int>()
print(a.sum())
