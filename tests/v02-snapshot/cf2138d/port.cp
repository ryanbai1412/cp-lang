// Source: Codeforces 2138 D, Antiamuny and Slider Movement,
//   https://codeforces.com/contest/2138/problem/D
// Problem: multitest (sum n, sum q <= 5000): n sliders at increasing positions a_i on a track
//   1..m (m <= 1e9); q operations "move slider i to x", pushing neighbours to keep them
//   distinct and ordered. For each slider print the sum of its final position over all q!
//   orders of the operations, modulo 1e9+7.
// Solution: Shift a_i by -i so pushes keep a non-decreasing order. For slider i and a threshold
//   y, ops at or left of i with value >= y force a'_i >= y, ops at or right of i with value < y
//   force a'_i < y; in a random order the last of these decides, so P(a'_i >= y) =
//   right / (left + right), or [a_i >= y] if none. Sweep y over the sorted op values and
//   integrate to get E[a'_i]; undo the shift and multiply by q!. O(n q) per test.

type Z = Mod<1_000_000_007>
let comb = Comb<Z>()

fn span_expect(lo: Int, hi: Int, ai: Int, num_left: Int, num_right: Int) -> Z:
    if hi <= lo: return 0
    if num_left + num_right == 0: Z(min(max(ai, lo), hi) - lo)
    else: Z(hi - lo) * Z(num_right) * comb.inv(num_left + num_right)

@!multitest
let (n, l, q): (1..=5000, 1..=1_000_000_000, 1..=5000) = read()
domain I = 0..n
let raw_a: [I]Int = read()
let a = [I] i => raw_a[i] - 1 - i.ord
let raw_ops: [0..q](Int, Int) = read()
let ops = [(val: x - i, pos: I(i - 1)) for (i, x) in raw_ops].sorted()

let fact_q = comb.fact(q)
var out: [I]Z = 0
for i in I:
    var num_left = 0
    var num_right = [1 for (_, pos) in ops if pos <= i].sum()
    var ans: Z = 0
    var lo = 0
    for (hi, pos) in ops:
        ans += span_expect(lo, hi, a[i], num_left, num_right)
        lo = hi
        if pos <= i: num_right -= 1
        if pos >= i: num_left += 1
    ans += span_expect(lo, l, a[i], num_left, num_right)
    out[i] = (ans + Z(i.ord + 1)) * fact_q

print(out)
