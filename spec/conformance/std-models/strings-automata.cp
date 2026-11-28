// expect: ok, output "ok"
// input: "1 300\n"
// model: Trie
// model: AhoCorasick
// model: DynamicAhoCorasick
// model: PalindromicTree
// The word automata on short random words over `ab`: a trie and an online pattern set against
// a plain multiset of words, a fresh Aho-Corasick automaton against the pattern prefixes, and a
// palindromic tree grown and shrunk at the back against every palindromic substring.
// Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
use std.strings.{Trie, AhoCorasick, DynamicAhoCorasick, PalindromicTree}

fn cut(s: Str, lo: Int, hi: Int) -> Str = Str([s[j] for j in lo..hi])
fn at(s: Str, i: Int, p: Str) -> Bool = 0 <= i and i + p.len <= s.len and cut(s, i, i + p.len) == p
fn suf(q: Str, w: Str) -> Bool = at(q, q.len - w.len, w)
fn rev(s: Str) -> Str = Str([s[s.len - 1 - j] for j in 0..s.len])
fn pal(s: Str) -> Bool = s == rev(s)
fn occ(s: Str, w: Str) -> Int = (0..=s.len).filter(i => at(s, i, w)).count()
fn uniq(xs: List<Str>) -> List<Str>:
    let ys = xs.sorted()
    [ys[i] for i in 0..ys.len if i == 0 or ys[i] != ys[i - 1]]
fn word(mut rng: Rng, hi: Int) -> Str = Str(["ab"[rng.int(0, 1)] for _ in 0..rng.int(0, hi)])
fn walk(ac: AhoCorasick<Char>, x: Str) -> Int:
    var st = ac.root
    for c in x: st = ac.step(st, c)
    st
fn lpps(p: Str) -> Str = [cut(p, i, p.len) for i in 1..=p.len if pal(cut(p, i, p.len))][0]
fn bdiff(p: Str) -> Int = p.len - lpps(p).len
fn series(p: Str) -> ?Str:
    let q = lpps(p)
    if q == "": return None
    if bdiff(q) != bdiff(p): Some(q) else: series(q)
fn pstate(t: PalindromicTree<Char>, p: Str) -> ?Int:
    if p == "": return Some(1)
    let h = p.len / 2
    var st: ?Int = if p.len % 2 == 1: t.step(0, p[h]) else: Some(1)
    for j in (if p.len % 2 == 1: h + 1 else: h)..p.len:
        if let x = st: st = t.step(x, p[j])
    st

let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var bad = ""
var trie = Trie<Char>()
var bag = List<Str>()
var made = [""]
var dyn = DynamicAhoCorasick<Char>()
var pats = List<Str>()
var pt = PalindromicTree<Char>()
var cur = ""
for k in 0..n:
    let w = word(mut rng, 3)
    if rng.chance(0.02):
        trie.clear()
        bag = []
        made = [""]
        if trie.nodes != 1 or trie.len != 0 or not trie.is_empty: bad = "op {k}: Trie.clear"
    elif bag.len == 0 or rng.chance(0.55):
        let v = trie.add(w)
        bag.push(w.clone())
        made = uniq(made + [cut(w, 0, j) for j in 0..=w.len])
        if trie.find(w) != Some(v): bad = "op {k}: Trie.add({w})"
    else:
        let x = if rng.chance(0.7): bag[rng.int(0, bag.len - 1)] else: w
        let i = (0..bag.len).filter(j => bag[j] == x).min()
        if trie.remove(x) != (i != None): bad = "op {k}: Trie.remove({x})"
        if let j = i: bag = [bag[y] for y in 0..bag.len if y != j]
    if bad != "": break
    let q = word(mut rng, 3)
    let sorted = bag.sorted()
    let kk = rng.int(0, bag.len)
    let np = bag.filter(x => at(x, 0, q)).count()
    let nc = bag.filter(x => x == q).count()
    if trie.root != 0 or trie.nodes != made.len or trie.len != bag.len or trie.is_empty != (bag.len == 0): bad = "op {k}: Trie.nodes/len/is_empty"
    elif trie.count(q) != nc or trie.contains(q) != (nc > 0) or trie.count_prefix(q) != np: bad = "op {k}: Trie.count({q})"
    elif (trie.find(q) == None) != (np == 0): bad = "op {k}: Trie.find({q})"
    elif np > 0 and (trie.ends_at(trie.find(q)!) != nc or trie.pass_at(trie.find(q)!) != np): bad = "op {k}: Trie.ends_at/pass_at({q})"
    elif q.len > 0 and np > 0 and trie.step(trie.find(cut(q, 0, q.len - 1))!, q[q.len - 1]) != trie.find(q): bad = "op {k}: Trie.step({q})"
    elif trie.kth(kk) != (if kk < bag.len: Some([c for c in sorted[kk]]) else: None): bad = "op {k}: Trie.kth({kk})"
    elif trie.count_less(q) != bag.filter(x => x < q).count(): bad = "op {k}: Trie.count_less({q})"
    if bad != "": break
    if pats.len == 0 or rng.chance(0.6):
        dyn.add(w.clone())
        pats.push(w)
    else:
        let j = rng.int(0, pats.len - 1)
        dyn.remove(pats[j])
        pats = [pats[y] for y in 0..pats.len if y != j]
    let text = word(mut rng, 8)
    if dyn.len != pats.len or dyn.is_empty != (pats.len == 0) or dyn.count_matches(text) != sum(occ(text, p) for p in pats): bad = "op {k}: DynamicAhoCorasick.count_matches({text})"
    if bad != "": break
    let ps = [word(mut rng, 3) for _ in 0..rng.int(1, 4)]
    let ac = AhoCorasick<Char>(ps)
    let pre = uniq([cut(p, 0, j) for p in ps, j in 0..=p.len])
    let sof = (x: Str) => walk(ac, x)
    let best = (x: Str) => [cut(x, i, x.len) for i in 0..=x.len if pre.contains(cut(x, i, x.len))][0]
    let byend = (x: Str) => (0..ps.len).filter(p => suf(x, ps[p])).sorted(by: p => (-ps[p].len, p))
    var hits = List<(at: Int, pattern: Int)>()
    var bfs = [""]
    var lvl = [""]
    while lvl.len > 0:
        lvl = [x + "{c}" for x in lvl, c in "ab" if pre.contains(x + "{c}")]
        bfs = bfs + lvl
    for e in 0..=text.len:
        for p in byend(cut(text, 0, e)): hits.push((at: e - ps[p].len, pattern: p))
    if ac.root != 0 or ac.len != pre.len or ac.fail_order() != bfs.map(sof).collect(): bad = "op {k}: AhoCorasick.len/fail_order"
    elif ac.find_all(text).collect() != hits or ac.counts(text) != [occ(text, p) for p in ps]: bad = "op {k}: AhoCorasick.find_all({text})"
    elif (0..ps.len).any(p => ac.pattern_state(p) != sof(ps[p])): bad = "op {k}: AhoCorasick.pattern_state"
    if bad != "": break
    for x in pre:
        let st = sof(x)
        let up = [cut(x, i, x.len) for i in 1..=x.len if ps.contains(cut(x, i, x.len))]
        if ac.link(st) != (if x == "": 0 else: sof(best(cut(x, 1, x.len)))): bad = "op {k}: AhoCorasick.link({x})"
        elif ac.exit_link(st) != (if up.is_empty: None else: Some(sof(up[0]))): bad = "op {k}: AhoCorasick.exit_link({x})"
        elif ac.words_at(st) != byend(x) or ac.hit(st) != (byend(x).len > 0): bad = "op {k}: AhoCorasick.words_at({x})"
        elif "ab".any(c => ac.step(st, c) != sof(best(x + "{c}"))): bad = "op {k}: AhoCorasick.step({x})"
        if bad != "": break
    if bad != "": break
    if cur.len >= 8 or rng.chance(0.05):
        pt = PalindromicTree<Char>()
        cur = ""
    elif cur.len > 0 and rng.chance(0.3):
        pt.pop()
        cur = cut(cur, 0, cur.len - 1)
    else:
        let c = "ab"[rng.int(0, 1)]
        cur += "{c}"
        if pt.push(c) != pt.last_state: bad = "op {k}: PalindromicTree.push"
    let m = cur.len
    let pals = uniq([cut(cur, l, r) for l in 0..m, r in l + 1..=m if pal(cut(cur, l, r))])
    var dp = [0]
    for i in 1..=m: dp.push((0..i).filter(j => pal(cut(cur, j, i))).map(j => dp[j] + 1).min()!)
    if pt.distinct != pals.len or pt.len != pals.len + 2 or pt.min_palindrome_partition() != dp: bad = "op {k}: PalindromicTree.distinct of {cur}"
    elif m > 0 and (pt.pal_len(pt.last_state) != lpps("x" + cur).len or pt.at(m - 1) != cur[m - 1]): bad = "op {k}: PalindromicTree.last_state of {cur}"
    elif pt.contains(q) != (q == "" or pals.contains(q)): bad = "op {k}: PalindromicTree.contains({q}) of {cur}"
    if bad != "": break
    let counts = pt.occ_counts()
    for p in pals:
        let st = pstate(pt, p)
        let up = lpps(p)
        if st == None or pt.pal_len(st!) != p.len or counts[st!] != occ(cur, p): bad = "op {k}: PalindromicTree state of {p} in {cur}"
        elif pt.link(st!) != pstate(pt, up)! or pt.diff(st!) != bdiff(p): bad = "op {k}: PalindromicTree.link({p}) in {cur}"
        elif pt.series_link(st!) != (if let s = series(p): pstate(pt, s)! else: 1): bad = "op {k}: PalindromicTree.series_link({p}) in {cur}"
        elif "ab".any(c => pt.step(st!, c) != (if pals.contains("{c}" + p + "{c}"): pstate(pt, "{c}" + p + "{c}") else: None)): bad = "op {k}: PalindromicTree.step({p}) in {cur}"
        if bad != "": break
    if bad != "": break
    if m > 0:
        var f = PalindromicTree<Char>.of(cut(cur, 1, m))
        let front = [cut(cur, 0, j) for j in 1..=m if pal(cut(cur, 0, j))].last!
        let st = f.push_front(cur[0])
        if f.pal_len(st) != front.len or f.distinct != pals.len: bad = "op {k}: PalindromicTree.push_front of {cur}"
    if bad != "": break
print(if bad == "": "ok" else: bad)
