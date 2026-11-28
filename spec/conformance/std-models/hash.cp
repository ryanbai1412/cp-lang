// expect: ok, output "ok"
// input: "1 3000\n"
// model: HashMap
// model: HashSet
// HashMap<Int, Int> against a List<?Int> by key and HashSet<Int> against a List<Bool>: seeded
// random set, add-assign, remove, insert, clear, reserve and rebuilds from pairs and from a list,
// with get, contains, len, is_empty, items, keys, values and == compared after each op. 64 keys
// make the table grow and wrap its probes. Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}

let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
let size = 64
var h = HashMap<Int, Int>()
var a: List<?Int> = List.filled(size, None)
var s = HashSet<Int>()
var b = List.filled(size, false)
var bad = ""
for k in 0..n:
    let op = rng.int(0, 9)
    let i = rng.int(0, size - 1)
    let v = rng.int(-5, 5)
    if op <= 1:
        h[i] = v
        a[i] = Some(v)
    elif op == 2:
        h[i] += v
        a[i] = Some((a[i] ?? 0) + v)
    elif op == 3:
        if h.remove(i) != a[i]: bad = "op {k}: remove {i}"
        a[i] = None
    elif op == 4:
        if s.insert(i) == b[i]: bad = "op {k}: insert {i}"
        b[i] = true
    elif op == 5:
        if s.remove(i) != b[i]: bad = "op {k}: set remove {i}"
        b[i] = false
    elif op == 6:
        if rng.int(0, 3) == 0:
            h.clear()
            s.clear()
            a = List.filled(size, None)
            b = List.filled(size, false)
        else:
            h.reserve(rng.int(-1, 100))
    elif op == 7:
        let ps = [(rng.int(0, size - 1), rng.int(-5, 5)) for _ in 0..rng.int(0, 12)]
        h = HashMap(ps)
        a = List.filled(size, None)
        for (key, w) in ps: a[key] = Some(w)
    else:
        let xs = [rng.int(0, size - 1) for _ in 0..rng.int(0, 12)]
        s = HashSet(xs)
        b = List.filled(size, false)
        for x in xs: b[x] = true
    if bad != "": break
    let live = [j for j in 0..size if a[j] != None]
    let members = [j for j in 0..size if b[j]]
    if h.len != live.len or h.is_empty != (live.len == 0):
        bad = "op {k}: len {h.len} want {live.len}"
    elif s.len != members.len or s.is_empty != (members.len == 0):
        bad = "op {k}: set len {s.len} want {members.len}"
    else:
        for j in 0..size:
            if h.get(j) != a[j] or (j in h) != (a[j] != None) or (j in s) != b[j]:
                bad = "op {k}: key {j}"
        let walked = [p for p in h.items].sorted()
        if walked != [(j, a[j]!) for j in live]: bad = "op {k}: items"
        if [x for x in h.keys].sorted() != live: bad = "op {k}: keys"
        if [x for x in h.values].sorted() != [a[j]! for j in live].sorted(): bad = "op {k}: values"
        if [x for x in s.items].sorted() != members: bad = "op {k}: set items"
        var g = HashMap<Int, Int>()
        for j in live.reversed(): g[j] = a[j]!
        let t = HashSet(members.reversed())
        if not (g == h) or not (t == s): bad = "op {k}: =="
        g[size] = 0
        if g == h: bad = "op {k}: == with an extra key"
    if bad != "": break
print(if bad == "": "ok" else: bad)
