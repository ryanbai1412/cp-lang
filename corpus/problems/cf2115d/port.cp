// Source: Codeforces 2115D (Gellyfish and Forget-Me-Not),
//   https://codeforces.com/contest/2115/problem/D
// limits: 2000 ms, 1000 MB
// Problem: x = 0; in round i = 1..n player c_i (0 minimises, 1 maximises the final x) sets
//   x ^= a_i or x ^= b_i, with a_i, b_i < 2^60. Print the final x under optimal play.
//   Multitest, sum n <= 1e5.
// Solution: start from x = xor of all a_i; round i may also xor in d_i = a_i ^ b_i. Go back
//   from the last round with a reduced XOR basis of later d's: if d_i reduces to 0, later
//   players can undo it; else its new pivot bit is touched by no other basis vector, so player
//   c_i alone sets that bit. Then apply each pivot vector iff it helps its owner: O(60 n).

@!multitest
let n: 1..=100_000 = read()
let a: [0..n]0..1 << 60 = read()
let b: [0..n]0..1 << 60 = read()
let c: Str = read()

var x = fold(^, a)

var basis: [0..60]Int = 0
var maximize: [0..60]Bool = false
for i in (0..n).rev:
    var v = a[i] ^ b[i]
    for bit in (0..60).rev: v min= v ^ basis[bit]
    if v != 0:
        for bit in 0..60: basis[bit] min= basis[bit] ^ v
        let bit = bit_width(v) - 1
        basis[bit] = v
        maximize[bit] = c[i] == '1'

for bit in 0..60 if basis[bit] != 0:
    if maximize[bit]: x max= x ^ basis[bit]
    else: x min= x ^ basis[bit]
print(x)
