// Source: Codeforces 2187D (Cool Problem), https://codeforces.com/contest/2187/problem/D
// limits: 4000 ms, 256 MB
// Problem: integers x, y <= 1e6 and a string s over 0, 1, ? (n <= 1e5). With c_0 = 0,
//   c_i = x + c_{i-1} on '0' and y - c_{i-1} on '1', f = sum c_i. Print the sum of the distinct
//   values f takes over all completions of s, mod 998244353. Multitest, sum n <= 1e5.
// Solution: the prefix before the first '?' is fixed (state c0, sum f). After it every completion
//   scores like a canonical shape 0 0^k 1^(m-k) or 1 0^k 1^(m-k), indexed by an end position in
//   -m..=m + 1, scored in O(1) by closed form. Scanning the rest keeps the reachable ends as
//   {2i + p0, 2i + p1} over subset sums i of half-gaps added at each '?' (they total <= m), so a
//   bitset subset sum lists them; the distinct scores go into a set and are summed.
// Bounds: n: 1..=100_000; x, y: 1..=1_000_000

type Z = Mod<998_244_353>

@!multitest
let (n, x, y): (1..=100_000, 1..=1_000_000, 1..=1_000_000) = read()
let s: Str = read()
fn step(c: Int, ch: Char) -> Int = if ch == '0': x + c else: y - c

let first_q = s.find('?')
let cut = first_q ?? s.len
let head = s[..cut]
var c0 = 0
var f = 0
for ch in head:
    c0 = step(c0, ch)
    f += c0
let q = first_q ?? return print(Z(f))

let rest = s[q + 1..]
var (p0, p1) = (1, 0)
var gaps: List<Int> = []
for ch in rest:
    match ch:
        '0' => (p0, p1) = (p0 + 1, p1 - 1)
        '1' => (p0, p1) = (p1, p0)
        _ =>
            gaps.push(abs(p0 - p1 + 1) / 2)
            p1 = min(p0, p1 - 1)
            p0 = p1 + 1

let m = rest.len
domain Half = 0..=m
let reach = Bitset<Half>.subset_sums(gaps)

fn walk_from(start: Int, k: Int) -> Int:
    let last = start + k * x
    let zeros = (k + 1) * start + k * (k + 1) / 2 * x
    let ones = (m - k) / 2 * last + (m - k + 1) / 2 * (y - last)
    zeros + ones

fn total(end: Int) -> Int =
    if end >= 1: f + walk_from(x + c0, end - 1) else: f + walk_from(y - c0, -end)

var cool = Set<Int>()
for i in reach:
    for p in [p0, p1]: cool.add(total(2 * i.ord + p))
print(sum(Z(v) for v in cool))
