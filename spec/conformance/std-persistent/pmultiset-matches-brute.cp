// expect: ok, output "YES"
// rule: std.persistent.PMultiset
// Random `with`/`without` on many versions over a small domain, checked against sorted lists:
// `count`, `len`, `==` and `cmp` (ascending and largest first) agree with the brute force, and old
// versions keep their contents.
use std.persistent.{PArena, PMultiset}
use std.rand.{Rng}
domain K = 0..7
var rng = Rng(5)
var ar = PArena<PMultiset<K>>()
var sets: List<PMultiset<K>> = [PMultiset.empty()]
// Version `i` came from `ops[i].0` by adding (or removing) key `ops[i].1`.
var ops: List<(Int, Int, Bool)> = [(-1, 0, true)]
// Version `i`'s elements, sorted, replayed from version 0 along its chain of parents.
fn brute(ops: List<(Int, Int, Bool)>, i: Int) -> List<Int>:
    var chain: List<Int> = []
    var j = i
    while j > 0:
        chain.push(j)
        j = ops[j].0
    var c = [0 for _ in 0..7]
    for t in chain.rev:
        let (_, k, add) = ops[t]
        if add: c[k] += 1
        else:
            if c[k] > 0: c[k] -= 1
    var out: List<Int> = []
    for k in 0..7:
        for _ in 0..c[k]: out.push(k)
    out
fn lex(a: List<Int>, b: List<Int>) -> Int:
    for i in 0..min(a.len, b.len):
        if a[i] != b[i]: return if a[i] < b[i]: -1 else: 1
    if a.len == b.len: 0 else: if a.len < b.len: -1 else: 1
var ok = true
for step in 0..400:
    let src = rng.int(0, sets.len - 1)
    let k = K(rng.int(0, 6))
    let add = rng.int(0, 2) > 0
    sets.push(if add: sets[src].with(mut ar, k) else: sets[src].without(mut ar, k))
    ops.push((src, k.ord, add))
for _ in 0..2000:
    let i = rng.int(0, sets.len - 1)
    let j = rng.int(0, sets.len - 1)
    let k = K(rng.int(0, 6))
    let a = brute(ops, i)
    let b = brute(ops, j)
    let ra = List(a.rev)
    let rb = List(b.rev)
    if sets[i].count(ar, k) != a.count(v => v == k.ord): ok = false
    if sets[i].len != a.len: ok = false
    if (sets[i] == sets[j]) != (a == b): ok = false
    if sets[i].cmp(ar, sets[j]) != lex(a, b): ok = false
    if sets[i].cmp(ar, sets[j], desc: true) != lex(ra, rb): ok = false
print(yes(ok))
