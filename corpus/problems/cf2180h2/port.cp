// Source: Codeforces 2180H2 (Bug Is Feature, Conditional Version),
//   https://codeforces.com/contest/2180/problem/H2
// limits: 2000 ms, 256 MB
// Problem: n series (sum n <= 2e5) of a < b < c <= l <= r <= 1e18, a, b, c an arithmetic
//   progression; series i is one game per x in [l, r]: raise one of a, b, c so they stay an
//   AP (any order) with common difference not decreasing and none above x. All games are
//   played together, Bug first, normal play: print Bug or Feature. Multitest, t <= 1e5.
// Solution: Sprague-Grundy, as in H1 (`cf2180h1`): a series with difference d = c - b
//   contributes prefix_xor(r - c + 1, d) ^ prefix_xor(l - c, d), the XOR of Grundy values
//   over x in [c, r] and [c, l). Closed forms by parity of d: odd d reduces to
//   g_hi = len / d + (len % d & 1) + 1, and popcount(g_hi) minus twice its bits at even
//   positions >= 2, mod 4, indexes ODD_D_GRUNDY = [0, 2, 3, 1]; even d gives 0 for even len,
//   else with g = len / d + 2 it is 0 when ctz(g) is odd, else 2 or 1 by popcount parity.
//   XOR everything: O(1) per series.

const EVEN_BITS_FROM_2 = 0x5555555555555554
const ODD_D_GRUNDY = [0, 2, 3, 1]

fn prefix_xor(len: Int, d: Int) -> Int:
    if d % 2 == 1:
        let g_hi = len / d + ((len % d) & 1) + 1
        let alt = popcount(g_hi) - 2 * popcount(g_hi & EVEN_BITS_FROM_2)
        ODD_D_GRUNDY[alt & 3]
    elif len % 2 == 0:
        0
    else:
        let g = len / d + 2
        if ctz(g) % 2 == 1: 0
        elif popcount(g) % 2 == 1: 2
        else: 1

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
