// fixed: k, d, lcm, base, lo
// let: popcount = \log n
use std.math.{crt_all, is_prime, primes_in, prime_count, floor_quotients, submasks, dirichlet_convolve, dirichlet_inv, sum_phi, sum_mobius, sum_multiplicative, sieve_multiplicative}
let n: Int = read()
var acc = 0
let eqs = [(5, 7) for i in 0..n]
let ones = [if i == 0: 0 else: 1 for i in 0..=n]
var pi = List.filled(n + 1, 0)
for v in 2..=n: pi[v] = pi[v - 1] + (if is_prime(v): 1 else: 0)
var vs: List<Int> = []
for f in floor_quotients(n): vs.push(f.q)
let ps = [pi[v] for v in vs.rev]
for _ in 0..32:
    if crt_all(eqs) != None: acc += 1 // timed: math.crt_all
    acc += sieve_multiplicative(n, (q, e, qe) => e + 1).len // timed: math.sieve_multiplicative
    for s in submasks(n - 1): acc += s // timed: math.submasks
    acc += dirichlet_convolve(ones, ones, n).len // timed: math.dirichlet_convolve
    acc += dirichlet_inv(ones, n).len // timed: math.dirichlet_inv
for _ in 0..128:
    acc += primes_in(0, n).len // timed: math.primes_in
    acc += sum_phi(n) // timed: math.sum_phi
    acc += sum_mobius(n) // timed: math.sum_mobius
for _ in 0..1024:
    acc += prime_count(n) // timed: math.prime_count
    acc += sum_multiplicative(n, (q, e, qe) => 1, ps) // timed: math.sum_multiplicative
print(acc)
