// Source: Codeforces 2268B (What a SauSaGe! It's All Meat),
//   https://codeforces.com/contest/2268/problem/B
// limits: 2000 ms, 256 MB
// Problem: n <= 2e5 values a_i < 16. An operation XORs a_i and a_{i+1} with the same 3k,
//   1 <= k <= 5. Before the q <= 2e5 point assignments a_p := x and after each, print the
//   most a_i that operations can make divisible by 3. Multitest, sums of n and q <= 2e5.
// Solution: 3, 6, 9, 12, 15 all have even popcount, so each a_i keeps its popcount parity;
//   every multiple of 3 below 16 has even popcount, and all even-popcount values can be
//   made multiples of 3 at once. So the answer is the count of even-popcount values,
//   kept in O(1) per update.
// Bounds: n: 2..=200_000; q: 0..=200_000; a: 0..=16 - 1; p: 1..=n; x: 0..16

fn even(x: 0..16) -> Int: Int(popcount(x) % 2 == 0)

@!multitest
let (n, q): (2..=200_000, 0..=200_000) = read()
var a: [1..=n]0..16 = read()

var good = sum(even(x) for x in a)
var out = [good]
for _ in 0..q:
    let (p, x): (1..=n, 0..16) = read()
    good += even(x) - even(a[p])
    a[p] = x
    out.push(good)
print(out)
