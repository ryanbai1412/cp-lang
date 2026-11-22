// Codeforces 2187 D: Cool Problem.
// The walk starts at c_0 = 0, updates c_i = x + c_{i-1} on '0' and
// c_i = y - c_{i-1} on '1', and scores sum(c_i).
//
// Up to the first '?', the prefix `head` is fixed, leaving state `c0` (0 when
// `head` is empty) and prefix sum `f`. After the first '?', every completion
// of length m + 1 has the same score as one of two canonical end-shapes indexed
// by an integer position `end` in -m..=m + 1:
//   - end = k + 1 >= 1: `0 0^k 1^(m - k)` starting at `start = x + c0`,
//   - end = -k <= 0:    `1 0^k 1^(m - k)` starting at `start = y - c0`.
// On `0^k 1^(m - k)` from `start`, the k + 1 zero-steps form the arithmetic
// progression `start, start + x, ..., last = start + k * x`, and the m - k
// one-steps alternate between `y - last` and `last`, giving the O(1) closed
// form in `walk_from(start, k)`.
//
// Tracking the reachable end positions through `rest` (the m chars after the
// first '?'): the first '?' seeds the opposite-parity pair `(p0, p1) = (1, 0)`.
//   - '0' drifts the two parities oppositely: `(p0, p1) = (p0 + 1, p1 - 1)`.
//   - '1' negates parity, swapping `(p0, p1) = (p1, p0)`.
//   - '?' unions `{p0 + 1, p1 - 1}` with `{p1, p0}`. Since `p0 - p1` is odd,
//     the union shifts the base pair `p1' = min(p0, p1 - 1), p0' = p1' + 1` by
//     either 0 or the even gap `2g = |p0 - p1 + 1|`, adding half-gap `g`.
// By induction, the reachable end positions are `{2i + p0, 2i + p1}` over all
// subset sums `i` of the half-gaps `g` (whose sum is at most m).

type Z = Mod<998_244_353>

@!multitest
let (n, x, y): (1..=100_000, 1..=1_000_000, 1..=1_000_000) = read()
let s: Str = read()
fn step(c: Int, ch: Char) -> Int = if ch == '0': x + c else: y - c

let first_q = s.find('?')
let head = if let q = first_q: s[..q] else: s
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

let cool = Set<Int>()
for i in reach:
    for p in [p0, p1]: cool.add(total(2 * i.ord + p))
print(sum(Z(v) for v in cool))
