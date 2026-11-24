// Source: Codeforces 2159C (Twin Polynomials), https://codeforces.com/contest/2159/problem/C
// limits: 2000 ms, 256 MB
// Problem: f = sum a_i x^i of degree n <= 4e5 (a_n != 0) is cool when it equals
//   g = sum i x^(a_i). Some a_i are given (<= 1e9), the rest (always a_0 and a_n) are -1.
//   Count the cool completions mod 1e9+7. Multitest, sum n <= 4e5.
// Solution: f = g iff for i >= 1 either a_i = 0 or a_i = j with a_j = i (a pairing of the
//   nonzero indices, fixed points allowed); a_0 then follows. Fill in the partners the given
//   values force (a clash or a value > n gives 0). Of the `free` open indices in 1..n-1 choose
//   i to be nonzero and pair them, with n if still open, in invs[i + extra] ways, where
//   invs[k] = invs[k-1] + (k-1) invs[k-2]. Sum over i: O(n).

const MOD = 1_000_000_007
type Z = Mod<MOD>
let comb = Comb<Z>()

@!multitest
let n: 1..=400_000 = read()
var a: [0..=n]?Int = read(none: -1)

for i in 1..=n if let v = a[i]:
    if v > n:
        print(0)
        return
    if v != 0:
        a[v] ??= i
        if a[v] != i:
            print(0)
            return

var invs: [0..=n]Z = 1
for i in 2..=n: invs[i] = invs[i - 1] + invs[i - 2] * (i - 1)

let free = (1..n).count(i => a[i] == None)
let extra = Int(a[n] == None)
print(sum(comb.choose(free, i) * invs[i + extra] for i in 0..=free))
