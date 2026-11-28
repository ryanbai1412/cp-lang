// expect: ok, output "ok"
// input: "1 300\n"
// model: SuffixArray
// model: suffix_array
// model: SuffixTreeEdge
// model: suffix_tree_edges
// model: SuffixAutomaton
// model: lcs_all
// The suffix structures on a random string grown one letter at a time: the suffix array, its
// queries and suffix-tree edges against sorted suffixes and substrings, and the suffix automaton
// against end-position sets of every substring. Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
use std.strings.{SuffixArray, SuffixAutomaton, SuffixTreeEdge, suffix_tree_edges, lcs_all}

fn cut(s: Str, lo: Int, hi: Int) -> Str = Str([s[j] for j in lo..hi])
fn at(s: Str, i: Int, p: Str) -> Bool = 0 <= i and i + p.len <= s.len and cut(s, i, i + p.len) == p
fn uniq(xs: List<Str>) -> List<Str>:
    let ys = xs.sorted()
    [ys[i] for i in 0..ys.len if i == 0 or ys[i] != ys[i - 1]]
fn subs(s: Str) -> List<Str> = [cut(s, l, r) for l in 0..s.len, r in l + 1..=s.len]
fn cmp(a: Str, b: Str) -> Int = if a < b: -1 elif a == b: 0 else: 1
fn ends(s: Str, w: Str) -> List<Int> = [i + w.len - 1 for i in 0..s.len if at(s, i, w)]

let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var bad = ""
var s = ""
var sam = SuffixAutomaton<Char>()
for k in 0..n:
    if s.len >= 8 or rng.chance(0.1):
        s = ""
        sam = SuffixAutomaton<Char>()
    let c = "abc"[rng.int(0, 2)]
    s += "{c}"
    sam.push(c)
    let m = s.len
    let p = Str(["abc"[rng.int(0, 2)] for _ in 0..rng.int(0, 3)])
    let sa = SuffixArray(s)
    let order = (0..m).sorted(by: i => cut(s, i, m))
    let lcpb = (i: Int, j: Int) => cut(s, i, m).common_prefix_len(cut(s, j, m))
    let ranks = [r for r in 0..m if at(s, order[r], p)]
    let below = [r for r in 0..m if cut(s, order[r], m) < p].len
    let all = uniq(subs(s))
    if sa.len != m or sa.is_empty or sa.sa != order or suffix_array(s).sa != order: bad = "op {k}: sa of {s}"
    elif (0..m - 1).any(r => sa.lcp[r] != lcpb(order[r], order[r + 1])): bad = "op {k}: lcp of {s}"
    elif sa.rank_range(p) != (if ranks.is_empty: below..below else: ranks[0]..ranks.last! + 1): bad = "op {k}: rank_range({p}) of {s}"
    elif sa.find_all(p).collect() != [order[r] for r in ranks] or sa.count(p) != ranks.len: bad = "op {k}: find_all/count({p}) of {s}"
    elif sa.distinct() != all.len: bad = "op {k}: distinct of {s}"
    if bad != "": break
    let (i, j) = (rng.int(0, m), rng.int(0, m))
    let (l1, r1, l2, r2) = (rng.int(0, m - 1), 0, rng.int(0, m - 1), 0)
    let e1 = rng.int(l1, m)
    let e2 = rng.int(l2, m)
    let i0 = rng.int(0, m - 1)
    let len = rng.int(0, m - i0)
    let near = [r for r in 0..m if lcpb(order[r], i0) >= len]
    let (ra, rb) = (rng.int(0, m - 1), rng.int(0, m - 1))
    let rr = min(ra, rb)..max(ra, rb) + 1
    let lr = [lcpb(order[rr.start], order[x]) for x in rr].min()!
    if sa.lcp_between(i, j) != lcpb(i, j): bad = "op {k}: lcp_between({i}, {j}) of {s}"
    elif sa.compare(l1, e1, l2, e2) != cmp(cut(s, l1, e1), cut(s, l2, e2)): bad = "op {k}: compare of {s}"
    elif sa.lcp_range(i0, len) != near[0]..near.last! + 1: bad = "op {k}: lcp_range({i0}, {len}) of {s}"
    elif sa.lcp_of_ranks(rr) != lr: bad = "op {k}: lcp_of_ranks of {s}"
    elif rr.len >= 2 and sa.split(rr) != (rr.start + 1..rr.end).filter(x => sa.lcp[x - 1] == lr).min()!: bad = "op {k}: split of {s}"
    if bad != "": break
    let q = rng.int(0, all.len)
    let kd = sa.kth_distinct(q)
    if (kd == None) != (q == all.len) or (q < all.len and cut(s, kd!.start, kd!.end) != all[q]): bad = "op {k}: kth_distinct({q}) of {s}"
    let edges = suffix_tree_edges(s)
    if sum(e.edge_len for e in edges) != all.len: bad = "op {k}: suffix_tree_edges total of {s}"
    for e in edges:
        let w = cut(s, order[e.rank_range.start], order[e.rank_range.start] + e.parent_len + e.edge_len)
        if e.edge_len <= 0 or e.bottom_size != e.top_size + e.edge_len or sa.rank_range(w) != e.rank_range: bad = "op {k}: edge {w} of {s}"
    if bad != "": break
    let w = Str(["abc"[rng.int(0, 2)] for _ in 0..rng.int(0, 3)])
    let has = w.len == 0 or ends(s, w).len > 0
    let st = sam.find(w)
    if sam.root != 0 or sam.len > max(2, 2 * m - 1) or sam.contains(w) != has or (st == None) == has: bad = "op {k}: SuffixAutomaton.find({w}) of {s}"
    elif sam.distinct() != all.len or SuffixAutomaton<Char>.of(s).distinct() != all.len: bad = "op {k}: SuffixAutomaton.distinct of {s}"
    elif sam.step(0, c) != sam.find("{c}"): bad = "op {k}: SuffixAutomaton.step of {s}"
    if bad != "": break
    if w.len > 0 and has:
        let v = st!
        let cls = [u for u in all if ends(s, u) == ends(s, w)]
        let lens = [u.len for u in cls]
        if sam.longest(v) != lens.max()! or sam.shortest(v) != lens.min()!: bad = "op {k}: longest/shortest of {w} in {s}"
        elif sam.first_pos(v) != Some(ends(s, w)[0]) or sam.occ_counts()[v] != ends(s, w).len: bad = "op {k}: first_pos/occ of {w} in {s}"
        elif sam.longest(sam.link(v)!) + 1 != sam.shortest(v): bad = "op {k}: link of {w} in {s}"
    if bad != "": break
    let topo = sam.topo_order()
    let tree = sam.endpos_tree()
    if topo.sorted() != [x for x in 0..sam.len] or (1..topo.len).any(x => sam.longest(topo[x - 1]) > sam.longest(topo[x])) or sam.states().collect() != topo: bad = "op {k}: topo_order of {s}"
    elif sam.link(0) != None or (1..sam.len).any(v => not tree[sam.link(v)!].contains(v)) or sum(t.len for t in tree) != sam.len - 1: bad = "op {k}: endpos_tree of {s}"
    if bad != "": break
    let o = Str(["abc"[rng.int(0, 2)] for _ in 0..rng.int(0, 6)])
    var best = (len: 0, end: 0)
    for e in 1..=o.len:
        for l in 0..e:
            if e - l > best.len and ends(s, cut(o, l, e)).len > 0: best = (len: e - l, end: e)
    let kq = rng.int(0, all.len)
    let many = subs(s).sorted()
    let kq2 = rng.int(0, many.len)
    if sam.lcs(o) != best: bad = "op {k}: lcs({o}) of {s}"
    elif sam.kth_substring(kq) != (if kq < all.len: Some([x for x in all[kq]]) else: None): bad = "op {k}: kth_substring({kq}) of {s}"
    elif sam.kth_substring(kq2, distinct: false) != (if kq2 < many.len: Some([x for x in many[kq2]]) else: None): bad = "op {k}: kth_substring({kq2}, distinct: false) of {s}"
    if bad != "": break
    var g = SuffixAutomaton<Char>()
    g.add_word(s)
    g.add_word(o)
    var g2 = SuffixAutomaton<Char>.of(o)
    g2.reset()
    for x in s: g2.push(x)
    let both = uniq(subs(s) + subs(o))
    if g.distinct() != both.len or g2.distinct() != both.len or g.contains(w) != (has or ends(o, w).len > 0): bad = "op {k}: add_word/reset {s} {o}"
    let words = [s, o, Str(["abc"[rng.int(0, 2)] for _ in 0..rng.int(0, 6)])][0..rng.int(1, 3)]
    let common = [u.len for u in all if words.all(x => ends(x, u).len > 0)]
    if lcs_all(words) != (common.max() ?? 0): bad = "op {k}: lcs_all"
    if bad != "": break
print(if bad == "": "ok" else: bad)
