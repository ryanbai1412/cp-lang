// expect: ok, output "0 1 2 2 3 2 4\n0 1 2 3\n0 1 -1 -1 0 -1 1\n32 0\n32 1 0 -1 1 0\n10 27\n303963552392 212"
use std.math.{DirichletSum, Sieve, dirichlet_convolve, divisors, dirichlet_inv, sum_mobius, sum_multiplicative, sum_phi}
print(dirichlet_convolve([0, 1, 1, 1, 1, 1, 1], [0, 1, 1, 1, 1, 1, 1], 6))
print(dirichlet_convolve([0, 1, 2, 3], [0, 1, 0, 0], 3))
print(dirichlet_inv([0, 1, 1, 1, 1, 1, 1], 6))
var ds = DirichletSum<Int>(10, [0, 1, 1, 2, 2], m => m, m => m * (m + 1) / 2)
print(ds.sum(10), ds.sum(0))
print(sum_phi(10), sum_phi(1), sum_phi(0), sum_mobius(10), sum_mobius(1), sum_mobius(0))
print(sum_multiplicative(10, (p, e, pe) => 1, [0, 1, 2, 3, 4]), sum_multiplicative(10, (p, e, pe) => e + 1, [0, 2, 4, 6, 8]))
let sv = Sieve(3000)
var (bp, bm) = (0, 0)
for n in 0..=3000:
    if n >= 1:
        bp += sv.phi(n)
        bm += sv.mobius(n)
    assert sum_phi(n) == bp and sum_mobius(n) == bm
for n in 1..=400:
    var vs: List<Int> = []
    for i in 1..=n:
        if vs.is_empty or vs[vs.len - 1] != n / i: vs.push(n / i)
    var ps: List<Int> = []
    for v in vs.rev:
        var c = 0
        for k in 2..=v:
            if sv.is_prime(k): c += 2
        ps.push(c)
    var want = 0
    for k in 1..=n: want += divisors(k).len
    assert sum_multiplicative(n, (p, e, pe) => e + 1, ps) == want
print(sum_phi(1_000_000), sum_mobius(1_000_000))
