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
var lists: List<List<Int>> = [[]]
fn lex(a: List<Int>, b: List<Int>) -> Int:
    for i in 0..min(a.len, b.len):
        if a[i] != b[i]: return if a[i] < b[i]: -1 else: 1
    if a.len == b.len: 0 else: if a.len < b.len: -1 else: 1
var ok = true
for step in 0..400:
    let src = rng.int(0, sets.len - 1)
    let k = K(rng.int(0, 6))
    let add = rng.int(0, 2) > 0
    var l = lists[src]
    let s = if add: sets[src].with(mut ar, k) else: sets[src].without(mut ar, k)
    if add: l.push(k.ord)
    else:
        var at = -1
        for i in 0..l.len:
            if l[i] == k.ord and at < 0: at = i
        if at >= 0: l.remove(at)
    l.sort()
    sets.push(s)
    lists.push(l)
for _ in 0..3000:
    let i = rng.int(0, sets.len - 1)
    let j = rng.int(0, sets.len - 1)
    let k = K(rng.int(0, 6))
    let (a, b) = (lists[i], lists[j])
    let (ra, rb) = (a.rev.collect(), b.rev.collect())
    if sets[i].count(ar, k) != a.count(v => v == k.ord): ok = false
    if sets[i].len != a.len: ok = false
    if (sets[i] == sets[j]) != (a == b): ok = false
    if sets[i].cmp(ar, sets[j]) != lex(a, b): ok = false
    if sets[i].cmp(ar, sets[j], desc: true) != lex(ra, rb): ok = false
print(yes(ok))
