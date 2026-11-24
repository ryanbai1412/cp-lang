// Codeforces 2245 C — Suffix Minima XOR Permutation
//
// Build a permutation of `0..n` whose suffix minima XOR to `k ^ n` (so that
// the full-array XOR with `n` yields `k`). Let `b = bit_width(n) - 1` be the
// top bit of `n`: every value in `0..n` is `< 2 << b` (and `< 1 << b` when
// `n` is a power of two), so `want = k ^ n` outside that range is impossible.
//
// Otherwise place `tail` (`[0]`, `[want, 0]`, or `[n - 1, want ^ (n - 1), 0]`)
// at the back in reverse order and all remaining values in decreasing order
// in front; the suffix minima are then exactly `tail`, which XOR to `want`.

@!multitest
let (n, k): (1..=200_000, 0..=10 ** 9) = read()
let b = bit_width(n) - 1
var want = k ^ n
if want >= 2 << b or (want >= 1 << b and n.is_pow2):
    print("NO")
    return

var tail: List<Int> = []
if want >= 1 << b:
    want ^= n - 1
    tail.push(n - 1)
if want > 0: tail.push(want)
tail.push(0)
print("YES")
print([i for i in (0..n).rev if not tail.contains(i)] + tail.rev)
