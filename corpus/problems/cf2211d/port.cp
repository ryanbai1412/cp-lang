// Source: Codeforces 2211D (AND-array), https://codeforces.com/contest/2211/problem/D
// limits: 3000 ms, 256 MB
// Problem: f(a)_k is the sum of the ANDs over all k-element subsequences of a. Given
//   b_k = f(a)_k mod 1e9+7 for k = 1..=n (n <= 1e5), print any a with 0 <= a_i < 2^29 that
//   produces it. Multitest, sum of n <= 1e5.
// Solution: f(a)_k = sum over bits of 2^bit * C(cnt[bit], k), where cnt[bit] counts elements
//   with that bit. Go k = n down to 1: subtract the bits already known to have cnt > k; what is
//   left is the sum of 2^bit over bits with cnt == k (below 2^29 < mod, so read it directly).
//   The element output at step k holds every bit with cnt >= k, so each bit lands in exactly
//   cnt elements. O(29 n) with a factorial table.

const MOD = 1_000_000_007
const L = 29
type Z = Mod<MOD>
let comb = Comb<Z>()

@!multitest
let n: 1..=200_000 = read()
let b: [1..=n]Z = read()

var cnts: [0..L]Int = 0
var out: List<Int> = []
for i in (1..=n).rev:
    var val = b[i]
    var a = 0
    for bit in 0..L if cnts[bit] > 0:
        val -= (1 << bit) * comb.choose(cnts[bit], i)
        a += 1 << bit
    if val != 0:
        for bit in 0..L if val.v.bit(bit):
            cnts[bit] = i
            a += 1 << bit
    out.push(a)
print(out)
