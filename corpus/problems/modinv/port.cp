// Source: DMOJ modinv (Modular Multiplicative Inverse), https://dmoj.ca/problem/modinv
// limits: 1000 ms, 256 MB
// Problem: 0 <= N < M <= 1e19 with N coprime to M (they fit in u64, not i64; N = 0 forces
//   M = 1). Print the inverse of N modulo M, nonnegative.
// Solution: extended Euclid on (M, N). Its coefficients of N alternate in sign and their sizes
//   t obey t' = t_prev + q t, never above M, so U64 holds them with a sign flag: the inverse is
//   t when the last coefficient is positive, else M - t. O(log M).
// original: corpus/submissions/5034440.cpp
let x: U64 = read()
let m: U64 = read()
assert x < m <= U64(10) ** 19
var (r0, r1) = (m, x)
var (t0, t1) = (U64(0), U64(1))
var plus = true
while r1 > U64(1):
    let q = r0 / r1
    (r0, r1) = (r1, r0 % r1)
    (t0, t1) = (t1, t0 + q * t1)
    plus = not plus
assert r1 == U64(1) or r0 == U64(1)  // coprime; r1 = 0 only for N = 0, M = 1
print(if r1 == U64(0): U64(0) elif plus: t1 else: m - t1)
