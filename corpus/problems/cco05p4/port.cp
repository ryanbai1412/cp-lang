// Source: CCO 2005 P4 (Primed Sequences), https://dmoj.ca/problem/cco05p4
// limits: 4000 ms, 16 MB
// Problem: t <= 20 tests, each a sequence of n <= 1e4 positive integers <= 1e4. Print the
//   shortest run of consecutive elements, length >= 2, whose sum is prime (leftmost on ties),
//   as "Shortest primed subsequence is length x: ..." followed by its elements, or print
//   "This sequence is anti-primed." if there is none. Multitest.
// Solution: brute force over prefix sums: try lengths 2, 3, ... and for each every left end
//   in order, looking the window sum up in a sieve of the odd sums built once; the first hit
//   is the answer. O(1e8 log log 1e8) once, then O(n^2) windows per test in the worst case.
// original: corpus/submissions/4665767.cpp
// Bounds: n: 1..=1e4; a: 1..=1e4
// Window sums are at most 1e4 * 1e4. Sieve the odd ones once for every test, bit k standing
// for 2k + 1, so the Bitset fits in 16 MB.
let top = 100_000_000
domain Half = 0..=top / 2
var composite = Bitset<Half>.single(Half(0))
for p in 3..10_000 by 2 if not composite.contains(Half(p / 2)):
    for m in p * p..top by 2 * p: composite.add(Half(m / 2))
// An even s looks up bit 0, the number 1, which is set: no branch on the parity.
fn is_prime(s: Int) -> Bool = s == 2 or not composite.contains(Half((s & 1) * (s >> 1)))

@multitest
let n: 1..=1e4 = read()
let a: [n]1..=1e4 = read()
let pre = prefix(a)

for len in 2..=n:
    if let l = (0..=n - len).find(l => is_prime(pre.cut(l + len) - pre.cut(l))):
        print("Shortest primed subsequence is length {len}:", [a[i] for i in l..l + len])
        return
print("This sequence is anti-primed.")
