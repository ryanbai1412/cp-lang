// Codeforces 2187 D: Cool Problem.
// Up to the first '?', c and f are fixed. After it, every completion ends at a
// position on one line of candidate strings 0 0^k 1^(m-k) / 1 0^k 1^(m-k),
// and each later '?' either keeps or skips a gap of even length, so the
// reachable positions are 2i + p for p in the two parity offsets and i a
// subset sum of the half-gaps: a bitset knapsack. Three equal weights g
// reach what g + 2g reach, so pairs carry to 2g and the knapsack has
// O(sqrt n) items (the half-gaps sum to at most n).
type Z = Mod<998_244_353>
@!multitest
let n: 1..=100_000 = read()
let x: 1..=1_000_000 = read()
let y: 1..=1_000_000 = read()
let s: Str = read()
let q = s.find('?') ?? s.len
var c0 = 0
var f = 0
for ch in s[..q]:
    c0 = if ch == '0': x + c0 else: y - c0
    f += c0
if q == s.len:
    print(Z(f))
else:
    let rest = s[q + 1..]
    let m = rest.len
    var (p0, p1) = (1, 0)
    domain Half = 0..=m
    var copies: [Half]Int = 0
    for ch in rest:
        if ch == '0':
            (p0, p1) = (p0 + 1, p1 - 1)
        elif ch == '1':
            (p0, p1) = (p1, p0)
        else:
            copies[Half(abs(p0 - p1 + 1) / 2)] += 1
            p1 = min(p0, p1 - 1)
            p0 = p1 + 1
    var reach = Bitset<Half>()
    reach.add(Half(0))
    for g in Half if g.to_idx > 0:
        let pairs = (copies[g] - 1) / 2
        if pairs > 0:
            copies[g] -= 2 * pairs
            if let g2 = Half.try_at(2 * g.to_idx): copies[g2] += pairs
        for _ in 0..copies[g]: reach.union(reach.shifted(g.to_idx))
    domain Pos = -m..=m + 1
    var end_f: [Pos]Int = 0
    for (zero_first, start) in [(true, x + c0), (false, y - c0)]:
        var c = start
        var total = f
        for k in 0..=m:
            total += c
            let pos = if zero_first: 1 + k else: -k
            end_f[Pos(pos)] = total + (m - k) / 2 * c + (m - k + 1) / 2 * (y - c)
            c += x
    let cool = Set<Int>()
    for i in reach.each():
        for p in [p0, p1]: cool.add(end_f[Pos(2 * i.to_idx + p)])
    print(sum(Z(v) for v in cool))
