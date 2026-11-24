// expect: ok, output "ok"
// input: "1 300\n"
// model: XorTrie
// model: BitTrie
// XorTrie(5) and BitTrie<5> against a List of the stored values: seeded random adds, removes,
// global xors, absorbs and rare clears, with counts, extreme, k-th and mex xors and prefix walks
// compared by brute force after each op. Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
use std.bitset.{BitTrie, XorTrie}
let w = 5
let top = 1 << w
let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var t = XorTrie(w)
var a: List<Int> = []
var bt = BitTrie<5>.new()
var b: List<Int> = []
var bad = ""
for k in 0..n:
    let op = rng.int(0, 6)
    let x = rng.int(0, top - 1)
    if op == 0 or op == 1:
        t.add(x)
        a.push(x)
    elif op == 2:
        let want = a.contains(x)
        if t.remove(x) != want: bad = "op {k}: remove {x}"
        if want:
            var j = 0
            while a[j] != x: j += 1
            a.remove(j)
    elif op == 3:
        t.xor_all(x)
        a = [y ^ x for y in a]
    elif op == 4 and x < 2:
        t.clear()
        a = []
    elif op == 4:
        t.reserve(x)
    if op == 5 and x % 3 == 0:
        var o = if x % 2 == 0: BitTrie<5>.of(x) else: BitTrie<5>.new()
        var ob = if x % 2 == 0: [x] else: []
        for _ in 0..rng.int(0, 4):
            let y = rng.int(0, top - 1)
            o.add(y)
            ob.push(y)
        bt.absorb(mut o)
        b = b + ob
        if not o.is_empty: bad = "op {k}: absorbed trie not empty"
    elif op == 5:
        bt.add(x)
        b.push(x)
    if bad != "": break
    let q = rng.int(0, top - 1)
    let lim = rng.int(0, top)
    let r = rng.int(0, a.len + 1)
    let xs = [y ^ q for y in a].sorted()
    var mex = 0
    while xs.contains(mex): mex += 1
    var mex0 = 0
    while a.contains(mex0): mex0 += 1
    var want_walk: List<(Int, Int)> = []
    for j in (0..w).rev:
        let c = [y for y in b if y >> j == q >> j].len
        if c == 0: break
        want_walk.push((j, c))
    if t.len != a.len or t.is_empty != (a.len == 0): bad = "op {k}: len"
    elif t.count(q) != [y for y in a if y == q].len or t.contains(q) != a.contains(q): bad = "op {k}: count {q}"
    elif t.max_xor(q) != xs.last or t.min_xor(q) != xs.first: bad = "op {k}: extreme xor {q}"
    elif t.count_xor_less(q, lim) != [v for v in xs if v < lim].len: bad = "op {k}: count_xor_less {q} {lim}"
    elif t.kth_xor(q, r) != (if r < xs.len: Some(xs[r]) else: None): bad = "op {k}: kth_xor {q} {r}"
    elif t.mex_xor(q) != mex or t.mex_xor() != mex0: bad = "op {k}: mex_xor {q}"
    elif bt.len != b.len or bt.is_empty != (b.len == 0) or bt.count(q) != [y for y in b if y == q].len: bad = "op {k}: bit trie count {q}"
    elif [(Int(e.bit), e.count) for e in bt.walk(q)] != want_walk: bad = "op {k}: walk {q}"
    if bad != "": break
print(if bad == "": "ok" else: bad)
