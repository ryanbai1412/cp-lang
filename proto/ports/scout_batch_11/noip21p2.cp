// NOIP 2021 P2, Sequence: a sequence of n values from 0..=m is good when
// the sum of 2^a has at most k one-bits; its weight is the product of
// v[a]. The total weight of all good sequences, modulo 998244353.
// Go through the values in increasing order: choosing c copies of value
// `bit` adds c to the carry at that bit, and the carry's low bit is final.
// original: corpus/submissions/5445677.cpp
const P = 998244353
type Z = Mod<P>

let n: 1..=30 = read()
let m: 0..=100 = read()
let k: 1..=30 = read() where k <= n
let v: [0..=m]1..=P - 1 = read()

// Pascal's triangle: binom[a, b] = a choose b
var binom: [0..=n, 0..=n]Z = 0
for a in 0..=n:
    binom[a, 0] = 1
    for b in 1..=a: binom[a, b] = binom[a - 1, b - 1] + binom[a - 1, b]

fn ones(x: Int) -> Int = sum(1 for b in 0..6 if (x >> b) % 2 == 1)

// Weight of every way to fill the `left` open places with values >= bit,
// given the carry into `bit` and the one-bits already fixed below it.
@memo
fn ways(left: 0..=n, bit: 0..=m + 1, carry: 0..=n, fixed: 0..=n) -> Z:
    if left == 0: return if fixed + ones(carry) <= k: 1 else: 0
    if bit > m or fixed > k: return 0
    var total: Z = 0
    var weight: Z = 1                        // v[bit] ^ c
    for c in 0..=left:
        total += binom[left, c] * weight * ways(left - c, bit + 1, (carry + c) / 2, fixed + (carry + c) % 2)
        weight *= v[bit]
    total

print(ways(n, 0, 0, 0))
