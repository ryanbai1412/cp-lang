use std.math.{DirichletSum, Sieve}
let n: Int = read()
var acc = 0
let f = Sieve(n).phi_table()
for _ in 0..128:
    let ds = DirichletSum<Int>(16 * n, f, m => m, m => m * (m + 1) / 2) // timed: math.DirichletSum.new
    acc += ds.n
var ds = DirichletSum<Int>(16 * n, f, m => m, m => m * (m + 1) / 2)
acc += ds.sum(16 * n)
for i in 0..32 * n:
    acc += ds.sum(16 * n / (i % 64 + 1)) // timed: math.DirichletSum.sum
print(acc)
