// expect: ok, output "ok 300"
// rule: std.segtree.MergeSegs.merge
// Sets in one MergeSegs pool share nodes: merging leaves both inputs valid, copying a handle is O(1),
// and a position found dead is gone from every set holding it. Checked against plain lists.
use std.segtree.{MergeSeg, MergeSegs}
domain K = 0..40
var p = MergeSegs<K>()
var dead: [K]Bool = false
var sets: List<MergeSeg<K>> = []
var want: List<List<Int>> = []
var ok = 0
for _ in 0..300:
    let op = rand(0, 3)
    if op == 0 or sets.len < 2:
        let x = rand(0, 39)
        sets.push(p.singleton(K(x)))
        want.push([x])
    elif op == 1:
        let (i, j) = (rand(0, sets.len - 1), rand(0, sets.len - 1))
        sets.push(p.merge(sets[i], sets[j]))
        want.push(want[i] + want[j])
    else:
        dead[K(rand(0, 39))] = true
    let i = rand(0, sets.len - 1)
    var s = sets[i]
    let got = p.first_alive(mut s, x => not dead[x])
    let best = want[i].filter(x => not dead[K(x)]).min()
    if got.map(x => x.idx) == best: ok += 1
print("ok", ok)
