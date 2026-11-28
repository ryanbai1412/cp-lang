// expect: ok, output "ok"
// input: "1 300\n"
// model: FlatMap
// FlatMap<Int> against a List<?Int> indexed by key: seeded random set, remove, get_or_insert,
// clear, reserve, take, and absorb with and without a join, on keys spread over a wide range so
// slots collide and the table rehashes, with get, contains, len, items, keys and values compared
// after each op. Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
use std.hashmap.{FlatMap}

fn key(i: Int) -> Int = (i - 20) * 1_000_003

fn entries(a: List<?Int>) -> List<(Int, Int)>:
    var out: List<(Int, Int)> = []
    for i in 0..a.len:
        if let v = a[i]: out.push((key(i), v))
    out.sorted()

let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var m = FlatMap<Int>(0, cap: rng.int(0, 4))
var a: List<?Int> = List.filled(40, None)
var bad = ""
for k in 0..n:
    let op = rng.int(0, 8)
    let i = rng.int(0, 39)
    let v = rng.int(-9, 9)
    if op <= 2:
        m.set(key(i), v)
        a[i] = Some(v)
    elif op == 3:
        if m.remove(key(i)) != a[i]: bad = "op {k}: remove {key(i)}"
        a[i] = None
    elif op == 4:
        let want = a[i] ?? v
        if m.get_or_insert(key(i), v) != want: bad = "op {k}: get_or_insert {key(i)}"
        a[i] = Some(want)
    elif op == 5:
        if rng.int(0, 4) == 0:
            m.clear()
            a = List.filled(40, None)
        else:
            m.reserve(rng.int(-1, 60))
    elif op == 6:
        let t = m.take()
        if not m.is_empty or m.len != 0 or m.get(key(i)) != None: bad = "op {k}: take left entries"
        m = t
    else:
        var o = FlatMap<Int>(0)
        var b: List<?Int> = List.filled(40, None)
        for _ in 0..rng.int(0, 8):
            let j = rng.int(0, 39)
            let w = rng.int(-9, 9)
            o.set(key(j), w)
            b[j] = Some(w)
        let join = rng.int(0, 1) == 1
        if join: m.absorb(mut o, join: Some((x, y) => x * 10 + y)) else: m.absorb(mut o)
        if not o.is_empty: bad = "op {k}: absorb left other non-empty"
        for j in 0..40:
            if let w = b[j]:
                a[j] = if let x = a[j], join: Some(x * 10 + w) else: Some(w)
    if bad != "": break
    let want = entries(a)
    if m.len != want.len or m.is_empty != (want.len == 0):
        bad = "op {k}: len {m.len} want {want.len}"
    elif m.items.sorted() != want:
        bad = "op {k}: items {m.items.sorted()} want {want}"
    elif m.keys.sorted() != [e.0 for e in want] or m.values.sorted() != [e.1 for e in want].sorted():
        bad = "op {k}: keys or values"
    elif m.items.map(e => e.0).collect() != m.keys.collect() or m.items.map(e => e.1).collect() != m.values.collect():
        bad = "op {k}: items, keys and values disagree on order"
    elif m.get(key(i)) != a[i] or m.contains(key(i)) != (a[i] != None) or m.get(key(i) + 1) != None:
        bad = "op {k}: get {key(i)}"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
