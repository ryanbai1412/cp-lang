// Source: Codeforces 2245C (MEXOR), https://codeforces.com/contest/2245/problem/C
// Problem: given n <= 2e5 and k <= 1e9, build a permutation p of 0..n-1 whose prefix mexes
//   mex(p_0..p_i), 0 <= i < n, XOR to k, or print NO. Multitest, sum n <= 2e5.
// Solution: mex(p_0..p_i) is the minimum of p_(i+1)..; the last is n, so the suffix minima of
//   positions 1..n-1 must XOR to want = k ^ n. With b the top bit of n, NO if want >= 2^(b+1),
//   or want >= 2^b when n is a power of two. Else put the other values in decreasing order,
//   then 0, want (or 0, want ^ (n - 1), n - 1 when want has bit b): the suffix minima are
//   those values and zeros. O(n).

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
print([i for i in (0..n).rev if not tail.any(x => x == i)] + tail.rev)
