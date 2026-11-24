// expect: ok, output "ok"
// input: "1 300\n"
// model: Marks
// model: MarkMap
// model: SparseSet
// model: Interner
// model: Arena
// Marks, MarkMap, SparseSet, Interner and Arena against plain arrays and lists: seeded random
// adds, sets, removes, clears, interning and allocs over a 12-position domain, with every
// position, the length and the walk order compared after each op. Reads `seed n`; prints the
// first op that differs.
use std.rand.{Rng}
use std.marks.{Arena, Id, Interner, MarkMap, Marks, SparseSet}
fn pos(xs: List<Int>, x: Int) -> ?Int:
    for j in 0..xs.len:
        if xs[j] == x: return Some(j)
    None

let seed0 = read<Int>()
let n = read<Int>()
domain V = 0..12
var rng = Rng(seed0)
var m = Marks<V>()
var mm = MarkMap<V, Int>(0)
var ss = SparseSet<V>()
var t = Interner<Int>()
var ar = Arena<Int>(cap: 4)
var in_m = List.filled(12, false)
var in_mm: List<?Int> = List.filled(12, None)
var dense: List<Int> = []
var keys: List<Int> = []
var nodes: List<Int> = []
var bad = ""
for k in 0..n:
    let op = rng.int(0, 13)
    let i = rng.int(0, 11)
    let x = rng.int(-9, 30)
    let v = V(i)
    if op == 0:
        if m.add(v) == in_m[i]: bad = "op {k}: Marks.add({i})"
        in_m[i] = true
    elif op == 1:
        if m.remove(v) != in_m[i]: bad = "op {k}: Marks.remove({i})"
        in_m[i] = false
    elif op == 2 and rng.int(0, 3) == 0:
        m.clear()
        in_m = List.filled(12, false)
    elif op == 3:
        mm.set(v, x)
        in_mm[i] = Some(x)
    elif op == 4:
        if mm.remove(v) != in_mm[i]: bad = "op {k}: MarkMap.remove({i})"
        in_mm[i] = None
    elif op == 5 and rng.int(0, 3) == 0:
        mm.clear()
        in_mm = List.filled(12, None)
    elif op == 6:
        let was = dense.contains(i)
        if ss.add(v) == was: bad = "op {k}: SparseSet.add({i})"
        if not was: dense.push(i)
    elif op == 7:
        let p = pos(dense, i)
        if ss.remove(v) != (p != None): bad = "op {k}: SparseSet.remove({i})"
        if let j = p:
            dense[j] = dense[dense.len - 1]
            dense.pop()
    elif op == 8 and rng.int(0, 3) == 0:
        ss.clear()
        dense = []
    elif op == 9:
        let want = pos(keys, x) ?? keys.len
        if want == keys.len: keys.push(x)
        if t.id(x) != want: bad = "op {k}: Interner.id({x})"
    elif op == 10:
        if t.get(x) != pos(keys, x) or t.contains(x) != keys.contains(x): bad = "op {k}: Interner.get({x})"
        elif keys.len > 0 and t.key(i % keys.len) != keys[i % keys.len]: bad = "op {k}: Interner.key"
    elif op == 11 and rng.int(0, 3) == 0:
        if rng.int(0, 1) == 0:
            t.clear()
            keys = []
        else:
            t.reserve(x + 9)
    elif op == 12:
        if ar.alloc(x) != Id<Int>(nodes.len): bad = "op {k}: Arena.alloc"
        nodes.push(x)
    elif op == 13 and nodes.len > 0:
        let j = i % nodes.len
        if ar.get(Id<Int>(j)) != nodes[j]: bad = "op {k}: Arena.get({j})"
        ar.set(Id<Int>(j), x)
        nodes[j] = x
        if rng.int(0, 9) == 0:
            ar.clear()
            nodes = []
        elif rng.int(0, 9) == 0:
            ar.reserve(nodes.len + x + 9)
    if bad != "": break
    let want_m = [j for j in 0..12 if in_m[j]]
    let want_mm = [(j, in_mm[j]!) for j in 0..12 if in_mm[j] != None]
    if [Int(u) for u in m.each()] != want_m or m.len != want_m.len or m.is_empty != want_m.is_empty:
        bad = "op {k}: Marks {m} want {want_m}"
    elif any(m.contains(V(j)) != in_m[j] for j in 0..12):
        bad = "op {k}: Marks.contains"
    elif [(Int(u), y) for (u, y) in mm.items] != want_mm or mm.len != want_mm.len or mm.is_empty != want_mm.is_empty:
        bad = "op {k}: MarkMap {mm} want {want_mm}"
    elif any(mm.get(V(j)) != in_mm[j] or mm.contains(V(j)) != (in_mm[j] != None) for j in 0..12):
        bad = "op {k}: MarkMap.get"
    elif [Int(u) for u in ss.each()] != dense or ss.len != dense.len or ss.is_empty != dense.is_empty:
        bad = "op {k}: SparseSet {ss} want {dense}"
    elif any(ss.contains(V(j)) != dense.contains(j) for j in 0..12):
        bad = "op {k}: SparseSet.contains"
    elif t.each().collect() != keys or t.len != keys.len or t.is_empty != keys.is_empty:
        bad = "op {k}: Interner {t} want {keys}"
    elif [y for (_, y) in ar.items] != nodes or [Int(h.raw) for (h, _) in ar.items] != [j for j in 0..nodes.len]:
        bad = "op {k}: Arena {ar} want {nodes}"
    elif ar.len != nodes.len or ar.is_empty != nodes.is_empty:
        bad = "op {k}: Arena.len"
    if bad != "": break
print(if bad == "": "ok" else: bad)
