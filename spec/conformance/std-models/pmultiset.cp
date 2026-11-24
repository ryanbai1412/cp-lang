// expect: ok, output "ok"
// input: "1 300\n"
// model: PMultiset
// model: PArena
// PMultisets sharing one PArena against sorted lists: seeded random with and without on a
// random old multiset, with len, is_empty, count, == and cmp (both orders) checked against a
// random other one; and a PArena<Int> against a plain list (alloc, get, len, is_empty,
// reserve). Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
use std.persistent.{PArena, PMultiset}
use std.marks.{Id}
let seed0 = read<Int>()
let n = read<Int>()
domain M = 0..8
var rng = Rng(seed0)
fn lex(a: List<Int>, b: List<Int>) -> Int:
    for i in 0..min(a.len, b.len):
        if a[i] != b[i]: return if a[i] < b[i]: -1 else: 1
    if a.len == b.len: 0 elif a.len < b.len: -1 else: 1
var ar = PArena<PMultiset<M>>(cap: 16)
var ms: List<PMultiset<M>> = [PMultiset<M>.empty()]
var bs: List<List<Int>> = [[]]
var pool = PArena<Int>()
var ids: List<Id<Int>> = []
var vals: List<Int> = []
var bad = ""
for k in 0..n:
    let v = rng.int(0, ms.len - 1)
    let x = rng.int(0, 7)
    var b = bs[v].clone()
    if rng.int(0, 2) > 0:
        ms.push(ms[v].with(mut ar, M(x)))
        b.push(x)
    else:
        ms.push(ms[v].without(mut ar, M(x)))
        var at = -1
        for q in 0..b.len:
            if at < 0 and b[q] == x: at = q
        if at >= 0: b = b[..at] + b[at + 1..]
    b.sort()
    bs.push(b)
    let y = rng.int(-5, 30)
    if rng.int(0, 9) == 0: pool.reserve(ids.len + 4)
    ids.push(pool.alloc(y))
    vals.push(y)
    let (i, j) = (rng.int(0, ms.len - 1), rng.int(0, ms.len - 1))
    let (s, t) = (ms[i], ms[j])
    let w = rng.int(0, ids.len - 1)
    if s.len != bs[i].len or s.is_empty != (bs[i].len == 0): bad = "op {k}: len {i}"
    elif s.count(ar, M(x)) != bs[i].count(z => z == x): bad = "op {k}: count {i} {x}"
    elif (s == t) != (bs[i] == bs[j]): bad = "op {k}: == {i} {j}"
    elif s.cmp(ar, t) != lex(bs[i], bs[j]): bad = "op {k}: cmp {i} {j}"
    elif s.cmp(ar, t, desc: true) != lex(bs[i].rev.collect(), bs[j].rev.collect()): bad = "op {k}: cmp desc {i} {j}"
    elif pool.get(ids[w]) != vals[w] or pool.len != vals.len or pool.is_empty: bad = "op {k}: arena"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
