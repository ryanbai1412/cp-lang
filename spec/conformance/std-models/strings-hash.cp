// expect: ok, output "ok"
// input: "1 300\n"
// model: HashVal
// model: RollingHash
// model: RollingHash61
// model: Hash2D
// model: DynamicHash
// The polynomial hashes over a random string edited by push, pop and set: equal hashes against
// equal slices, combine and concat against the joined slice, lcp and lcs against a scan, and 2D
// rectangles against cell-by-cell comparison. Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
use std.strings.{Hash2D, DynamicHash, HashVal, RollingHash61}

domain R = 0..5
domain C = 0..5

fn cut(s: Str, lo: Int, hi: Int) -> Str = Str([s[j] for j in lo..hi])
fn lcp_at(s: Str, a: Int, b: Int) -> Int = cut(s, a, s.len).common_prefix_len(cut(s, b, s.len))
fn lcs_at(s: Str, a: Int, b: Int) -> Int = cut(s, 0, a).common_suffix_len(cut(s, 0, b))

let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var bad = ""
var s = ""
var h = RollingHash("")
var h61 = RollingHash61("")
var dh = DynamicHash("abababab")
var d = "abababab"
var grid = [[0 for _ in 0..5] for _ in 0..5]
var h2 = Hash2D<R, C>((r, c) => grid[r.ord][c.ord])
for k in 0..n:
    let op = rng.int(0, 9)
    if op <= 4 and s.len < 12:
        let c = "ab"[rng.int(0, 1)]
        s += "{c}"
        h.push(c)
        h61.push(c)
    elif op <= 6 and s.len > 0:
        s = cut(s, 0, s.len - 1)
        h.pop()
        h61.pop()
    elif op == 7:
        let i = rng.int(0, d.len - 1)
        let c = "ab"[rng.int(0, 1)]
        d = cut(d, 0, i) + "{c}" + cut(d, i + 1, d.len)
        dh.set(i, c)
    elif op == 8:
        grid = [[rng.int(0, 1) for _ in 0..5] for _ in 0..5]
        h2 = Hash2D<R, C>((r, c) => grid[r.ord][c.ord])
    let m = s.len
    let len = rng.int(0, m)
    let (l1, l2) = (rng.int(0, m - len), rng.int(0, m - len))
    let (r1, r2) = (l1 + len, l2 + len)
    let same = cut(s, l1, r1) == cut(s, l2, r2)
    let mid = rng.int(l1, r1)
    let (a, b) = (rng.int(0, m), rng.int(0, m))
    if h.len != m or h.is_empty != (m == 0) or h61.len != m or h61.is_empty != (m == 0): bad = "op {k}: len of {s}"
    elif (h.get(l1..r1) == h.get(l2..r2)) != same or (h.slice(l1, r1) == h.slice(l2, r2)) != same: bad = "op {k}: RollingHash {l1}..{r1} vs {l2}..{r2} of {s}"
    elif (h61.get(l1..r1) == h61.get(l2..r2)) != same: bad = "op {k}: RollingHash61 {l1}..{r1} vs {l2}..{r2} of {s}"
    elif h.all != h.get(0..m) or h.hash_of(cut(s, l1, r1)) != h.get(l1..r1) or h61.all != h61.get(0..m): bad = "op {k}: all/hash_of of {s}"
    elif h.combine(h.get(l1..mid), h.get(mid..r1), r1 - mid) != h.get(l1..r1) or h61.combine(h61.get(l1..mid), h61.get(mid..r1), r1 - mid) != h61.get(l1..r1): bad = "op {k}: combine at {mid}"
    elif h.concat([(hash: h.get(l1..mid), len: mid - l1), (hash: h.get(mid..r1), len: r1 - mid)]) != h.get(l1..r1): bad = "op {k}: concat at {mid}"
    elif h.lcp(a, b) != lcp_at(s, a, b) or h61.lcp(a, b) != lcp_at(s, a, b): bad = "op {k}: lcp({a}, {b}) of {s}"
    elif h.lcs(a, b) != lcs_at(s, a, b) or h61.lcs(a, b) != lcs_at(s, a, b): bad = "op {k}: lcs({a}, {b}) of {s}"
    if bad != "": break
    let hv = h.get(l1..r1)
    let rows = RollingHash([h.get(l1..r1), h.get(l2..r2)])
    if hv.hash_token != (hv.h1 << 31) | hv.h2: bad = "op {k}: HashVal.hash_token"
    elif (rows.get(0..1) == rows.get(1..2)) != same: bad = "op {k}: RollingHash of HashVal"
    if bad != "": break
    let dl = rng.int(0, d.len)
    let dr = rng.int(dl, d.len)
    let dl2 = rng.int(0, d.len - (dr - dl))
    let dr2 = dl2 + dr - dl
    let back = Str([d[dr - 1 - j] for j in 0..dr - dl])
    if dh.len != d.len or dh.is_empty or dh.all != dh.get(0..d.len): bad = "op {k}: DynamicHash len/all"
    elif (dh.get(dl..dr) == dh.get(dl2..dr2)) != (cut(d, dl, dr) == cut(d, dl2, dr2)): bad = "op {k}: DynamicHash.get {dl}..{dr} vs {dl2}..{dr2} of {d}"
    elif (dh.rev_get(dl..dr) == dh.get(dl2..dr2)) != (back == cut(d, dl2, dr2)): bad = "op {k}: DynamicHash.rev_get {dl}..{dr} of {d}"
    elif dh.is_palindrome(dl..dr) != (back == cut(d, dl, dr)): bad = "op {k}: DynamicHash.is_palindrome {dl}..{dr} of {d}"
    if bad != "": break
    let (hr, wc) = (rng.int(0, 4), rng.int(0, 4))
    let (ra, rb, ca, cb) = (rng.int(0, 4 - hr), rng.int(0, 4 - hr), rng.int(0, 4 - wc), rng.int(0, 4 - wc))
    let eq = (0..hr).all(i => (0..wc).all(j => grid[ra + i][ca + j] == grid[rb + i][cb + j]))
    let x = h2.rect(R(ra)..R(ra + hr), C(ca)..C(ca + wc))
    if (x == h2.rect(R(rb)..R(rb + hr), C(cb)..C(cb + wc))) != eq: bad = "op {k}: Hash2D {hr}x{wc} at ({ra}, {ca}) vs ({rb}, {cb})"
    elif (hr == 0 or wc == 0) and x != HashVal(0, 0): bad = "op {k}: Hash2D empty rect"
    if bad != "": break
print(if bad == "": "ok" else: bad)
