// expect: ok, output "ok"
// rule: std.tree.LazyEulerTourTree
use std.tree.{LazyEulerTourTree}
use std.algebra.{add}
domain V = 0..8
var x = 5
var et = LazyEulerTourTree<V, Int, Int>(add)
var val = [V] _ => 0
var linked = [V] _ => false
var bad = 0
fn comp(linked: [V]Bool, v: V) -> List<V>:
    var lo = v.ord
    while lo > 0 and linked[V(lo - 1)]: lo -= 1
    var hi = v.ord
    while hi < 7 and linked[V(hi)]: hi += 1
    [V(i) for i in lo..=hi]
for _ in 0..300:
    x = (x * 1103515245 + 12345) % 2147483648
    let i = x % 7
    x = (x * 1103515245 + 12345) % 2147483648
    let v = V(x % 8)
    let d = (x / 64) % 9 - 4
    if (x / 1024) % 3 == 0:
        if linked[V(i)]: et.cut(V(i), V(i + 1))
        else: et.link(V(i), V(i + 1))
        linked[V(i)] = not linked[V(i)]
    else:
        et.apply_tree(v, d)
        for w in comp(linked, v): val[w] += d
    if et.to_array() != val: bad += 1
if "{et}" != " ".join(["{val[w]}" for w in V]): bad += 1
print(if bad == 0: "ok" else: "{bad} mismatches")
