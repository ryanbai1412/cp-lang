// Source: Codeforces 2180H1, https://codeforces.com/contest/2180/problem/H1
// Problem: multitest; n <= 2e5 series (sum n <= 2e5) of an AP a < b < c <= l <= r <= 1e18.
//   Series i is one game per x in [l, r]: raise one of a, b, c so they stay an AP (any order),
//   all <= x. Bug moves first in the sum of all games and whoever cannot move loses. Print Bug
//   or Feature.
// Solution: by Sprague-Grundy Bug wins iff the XOR of all Grundy values is non-zero, and a
//   game's value depends only on d = c - b and the offset x - c. prefix_xor(len, d) is the XOR
//   over offsets 0..len in closed form: write d = 2^l * odd, collapse the offsets to
//   g_hi = len / odd + ((len % odd) & 1), and a pattern of period 2^(l+2) in g_hi gives the
//   bits. A series is prefix_xor(r - c + 1) ^ prefix_xor(l - c): O(1) each.

fn prefix_xor(len: Int, d0: Int) -> Int:
    let l = ctz(d0)
    let d = d0 >> l
    let half_period = 2 << l
    let period_mask = half_period - 1
    let g_hi = len / d + ((len % d) & 1)
    var res = Int(g_hi.bit(l + 1))
    let rem = g_hi & period_mask
    if rem == 0: return res
    res ^= 1
    if rem % 2 == 1:
        if rem.bit(l):
            let i = bit_width(half_period - rem) - 1
            res ^= if ((l ^ i) & 1) == 1: 2 else: 1
        else:
            let i = bit_width(rem) - 1
            res ^= if ((l ^ i) & 1) == 1: 0 else: 1
    res

@!multitest
let n: 1..=200_000 = read()
let qs: [0..n](
    a: 1..=10 ** 18,
    b: 1..=10 ** 18,
    c: 1..=10 ** 18,
    l: 1..=10 ** 18,
    r: 1..=10 ** 18,
) = read()
let ans = fold(^, prefix_xor(r - c + 1, c - b) ^ prefix_xor(l - c, c - b) for (_, b, c, l, r) in qs)
print(if ans != 0: "Bug" else: "Feature")
