// Codeforces 2115 D — Gellyfish and Forget-Me-Not
//
// Starting from `x = XOR_i a[i]`, at each turn i = 0..n - 1 player `c[i]`
// ('1' maximizes, '0' minimizes) may optionally XOR `x` by `d[i] = a[i] ^ b[i]`.
//
// Process turns backward from n - 1 down to 0, maintaining a reduced row-echelon
// XOR basis of all later moves. Reducing `d[i]` against later basis vectors
// leaves either 0 (later players can completely undo turn i, so turn i has no
// effect) or a new pivot vector `v` with highest bit `bit`: reducing existing
// basis vectors by `v` makes `bit` unique to `v`, so player `c[i]` uniquely
// decides bit `bit` of the final XOR (`max` for '1', `min` for '0').

@!multitest
let n: 1..=100_000 = read()
let a: [0..n]0..=10 ** 18 = read()
let b: [0..n]0..=10 ** 18 = read()
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
