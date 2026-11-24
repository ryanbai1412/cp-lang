// expect: ok, output "ok"
// input: "1 300\n"
// model: z_function
// model: prefix_function
// model: find_all
// model: shift_and
// model: find_all_wildcard
// model: period
// model: is_palindrome
// model: borders
// model: prefix_matches
// model: overlap
// model: lyndon_factorization
// model: min_rotation
// model: max_rotation
// model: KmpAutomaton
// model: manacher
// model: Palindromes
// model: lyndon_runs
// The exact-matching and periodicity helpers on short random strings over a tiny alphabet, each
// against its definition checked by direct comparison of slices. Reads `seed n`; prints the
// first op that differs.
use std.rand.{Rng}
use std.strings.{KmpAutomaton, max_rotation}

fn cut(s: Str, lo: Int, hi: Int) -> Str = Str([s[j] for j in lo..hi])
fn at(s: Str, i: Int, p: Str) -> Bool = i + p.len <= s.len and cut(s, i, i + p.len) == p
fn rev(s: Str) -> Str = Str([s[s.len - 1 - j] for j in 0..s.len])
fn lyndon(w: Str) -> Bool = w.len > 0 and (1..w.len).all(j => w < cut(w, j, w.len))
fn per(w: Str) -> Int = (1..=w.len).filter(d => (0..w.len - d).all(i => w[i] == w[i + d])).min() ?? 0

let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var bad = ""
for k in 0..n:
    let ab = ["ab", "abc", "a"][rng.int(0, 2)]
    let s = Str([ab[rng.int(0, ab.len - 1)] for _ in 0..rng.int(0, 10)])
    let p = Str([ab[rng.int(0, ab.len - 1)] for _ in 0..rng.int(0, 3)])
    let m = s.len
    let z = [s.common_prefix_len(cut(s, i, m)) for i in 0..m]
    let pf = [(0..=i).filter(j => at(cut(s, 0, i + 1), i + 1 - j, cut(s, 0, j))).max()! for i in 0..m]
    let occ = [i for i in 0..m if at(s, i, p)]
    if z_function(s) != z: bad = "op {k}: z_function({s})"
    elif prefix_function(s) != pf: bad = "op {k}: prefix_function({s})"
    elif s.find_all(p).collect() != occ: bad = "op {k}: find_all({s}, {p})"
    elif s.shift_and(p).collect() != occ: bad = "op {k}: shift_and({s}, {p})"
    if bad != "": break
    let w = Str([(if rng.chance(0.3): '?' else: c) for c in p])
    let wild = [i for i in 0..m if i + w.len <= m and (0..w.len).all(j => w[j] == '?' or w[j] == s[i + j])]
    let pm = [(0..=min(p.len, i + 1)).filter(j => at(s, i + 1 - j, cut(p, 0, j))).max()! for i in 0..m]
    let ov = (0..=min(m, p.len)).filter(j => at(s, m - j, cut(p, 0, j))).max()!
    if s.find_all_wildcard(w).collect() != wild: bad = "op {k}: find_all_wildcard({s}, {w})"
    elif period(s) != per(s): bad = "op {k}: period({s})"
    elif is_palindrome(s) != (s == rev(s)): bad = "op {k}: is_palindrome({s})"
    elif borders(s).collect() != [b for b in (1..m).rev if cut(s, 0, b) == cut(s, m - b, m)]: bad = "op {k}: borders({s})"
    elif s.prefix_matches(p) != pm: bad = "op {k}: prefix_matches({s}, {p})"
    elif overlap(s, p) != ov: bad = "op {k}: overlap({s}, {p})"
    if bad != "": break
    var fac: List<Int> = []
    var i = 0
    while i < m:
        fac.push(i)
        i += (1..=m - i).filter(j => lyndon(cut(s, i, i + j))).max()!
    let rots = [cut(s, j, m) + cut(s, 0, j) for j in 0..m]
    let lo: ?Int = if m == 0: None else: (0..m).filter(j => rots[j] == rots.min()!).min()
    let hi: ?Int = if m == 0: None else: (0..m).filter(j => rots[j] == rots.max()!).min()
    if lyndon_factorization(s) != fac: bad = "op {k}: lyndon_factorization({s})"
    elif min_rotation(s) != lo: bad = "op {k}: min_rotation({s})"
    elif max_rotation(s) != hi: bad = "op {k}: max_rotation({s})"
    if bad != "": break
    let kmp = KmpAutomaton<Char>(p)
    var st = 0
    var kmp_ok = kmp.len == p.len + 1 and (1..=p.len).all(j => kmp.link(j) == prefix_function(p)[j - 1]) and kmp.link(0) == 0
    for j in 0..m:
        st = kmp.step(st, s[j])
        if st != pm[j] and not (p.len == 0): kmp_ok = false
    if not kmp_ok: bad = "op {k}: KmpAutomaton({p}) on {s}"
    elif kmp.find_all(s).collect() != (if p.len == 0: [j for j in 0..=m] else: occ): bad = "op {k}: KmpAutomaton.find_all({s}, {p})"
    if bad != "": break
    let pal = manacher(s)
    let odd = [(1..=m).filter(r => i - r + 1 >= 0 and i + r <= m and is_palindrome(cut(s, i - r + 1, i + r))).max() ?? 0 for i in 0..m]
    let even = [(0..=m).filter(r => i - r >= 0 and i + r <= m and is_palindrome(cut(s, i - r, i + r))).max()! for i in 0..m]
    var best = (0, 0)
    for l in 0..m:
        for r in l + 1..=m:
            if r - l > best.1 - best.0 and is_palindrome(cut(s, l, r)): best = (l, r)
    let (ql, qr) = (rng.int(0, m), rng.int(0, m))
    let (l, r) = (min(ql, qr), max(ql, qr))
    if pal.len != m or pal.is_empty != (m == 0) or pal.odd != odd: bad = "op {k}: manacher({s}).odd"
    elif (0..m).any(i => pal.longest_at(i) != 2 * odd[i] - 1): bad = "op {k}: longest_at on {s}"
    elif (0..=m).any(i => pal.longest_between(i) != (if i == m: 0 else: 2 * even[i])): bad = "op {k}: longest_between on {s}"
    elif pal.is_palindrome(l..r) != is_palindrome(cut(s, l, r)): bad = "op {k}: Palindromes.is_palindrome({l}..{r}) on {s}"
    elif pal.count() != sum(odd) + sum(even): bad = "op {k}: count on {s}"
    elif pal.longest() != (l: best.0, r: best.1): bad = "op {k}: longest on {s}"
    if bad != "": break
    var runs: List<(l: Int, r: Int, period: Int)> = []
    for l in 0..m:
        for r in l + 2..=m:
            let d = per(cut(s, l, r))
            if r - l >= 2 * d and (l == 0 or s[l - 1] != s[l - 1 + d]) and (r == m or s[r] != s[r - d]): runs.push((l: l, r: r, period: d))
    if lyndon_runs(s) != runs: bad = "op {k}: lyndon_runs({s})"
    if bad != "": break
print(if bad == "": "ok" else: bad)
