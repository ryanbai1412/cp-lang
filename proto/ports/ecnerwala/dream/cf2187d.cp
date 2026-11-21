// Codeforces 2187 D: Cool Problem (dream). Before the first '?' the walk is
// fixed; after it the end positions are 2i + p, i a subset sum of half-gaps.
type Z = Mod<998_244_353>
@!multitest
let (n, x, y): (1..=1e5, 1..=1e6, 1..=1e6) = read()
let s: Str of "01?" = read()
fn step(c: Int, ch: Char) -> Int = if ch == '0': x + c else: y - c
let walk = s.scan(step, from: 0)                                 // c_1 .. c_n
let (head, rest) = s.split_first('?') ?? return print(Z(walk.sum()))
let (c0, f) = (walk[head.end] ?? 0, walk[..head.end].sum())
var (p0, p1) = (1, 0)
var gaps = Bag<0..=rest.len>()
for ch in rest: match ch:
    '0' => (p0, p1) += (1, -1)
    '1' => (p0, p1) = (p1, p0)
    '?' =>
        gaps.add(abs(p0 - p1 + 1) / 2)
        p1 = min(p0, p1 - 1)
        p0 = p1 + 1
let reach = subset_sums(gaps, upto: rest.len)                    // Bitset, carries equal weights
let m = rest.len
// The walk after the first '?' is one of two shapes, ending k + 1 or -k:
//   0 0^k 1^(m-k)  or  1 0^k 1^(m-k);  each run of 0s then alternating 1s is summed whole.
fn walk_from(c: Int, k: Int) -> Int =
    let zeros = sum(c + j * x for j in 0..=k)
    let ones = (m - k) / 2 * (c + k * x) + (m - k + 1) / 2 * (y - c - k * x)
    zeros + ones
fn total(end: Int) -> Int =
    if end >= 1: f + walk_from(x + c0, end - 1) else: f + walk_from(y - c0, -end)
print(sum(Z(v) for v in {total(2 * i + p) for i in reach for p in [p0, p1]}))
