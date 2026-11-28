// expect: ok, output "ok"
// input: "1 300\n"
// model: SortedMap
// SortedMap<Int, Int> against a List<?Int> over keys -8..=8: seeded random set, remove,
// pop_first, pop_last, drain, split_at with join, absorb with and without a join, clear, take
// and reserve, with get, contains, len, first, last, first_ge, first_gt, last_le, last_lt,
// rank, nth, first_true, last_true, items, each_rev, keys and values compared after each op.
// Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
use std.bag.{SortedMap}

fn entries(a: List<?Int>) -> List<(Int, Int)>:
    var out: List<(Int, Int)> = []
    for i in 0..a.len:
        if let v = a[i]: out.push((i - 8, v))
    out

fn opt(a: List<(Int, Int)>, i: Int) -> ?(Int, Int) = if 0 <= i and i < a.len: Some(a[i]) else: None

fn last_of(a: List<(Int, Int)>) -> ?(Int, Int) = opt(a, a.len - 1)

fn rev(a: List<(Int, Int)>) -> List<(Int, Int)> = [a[a.len - 1 - i] for i in 0..a.len]

fn at(a: List<?Int>, x: Int) -> ?Int = if -8 <= x and x <= 8: a[x + 8] else: None

let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var want: List<?Int> = List.filled(17, None)
var m: SortedMap<Int, Int> = SortedMap()
var bad = ""
for k in 0..n:
    let op = rng.int(0, 10)
    let x = rng.int(-8, 8)
    let v = rng.int(-9, 9)
    if op <= 2:
        m.set(x, v)
        want[x + 8] = Some(v)
    elif op == 3:
        if m.remove(x) != want[x + 8]: bad = "op {k}: remove {x}"
        want[x + 8] = None
    elif op == 4:
        let e = entries(want)
        if rng.int(0, 1) == 0:
            if m.pop_first() != opt(e, 0): bad = "op {k}: pop_first"
            if e.len > 0: want[e[0].0 + 8] = None
        else:
            if m.pop_last() != last_of(e): bad = "op {k}: pop_last"
            if e.len > 0: want[e[e.len - 1].0 + 8] = None
    elif op == 5:
        let side = rng.int(0, 2)
        let e = entries(want)
        var w: List<(Int, Int)> = []
        var got: List<(Int, Int)> = []
        if side == 0:
            got = m.drain(after: x, while: (key, val) => val < v)
            for p in e:
                if p.0 > x:
                    if p.1 >= v: break
                    w.push(p)
        elif side == 1:
            got = m.drain(before: x, while: (key, val) => val > v)
            for p in rev(e):
                if p.0 < x:
                    if p.1 <= v: break
                    w.push(p)
        else:
            got = m.drain(while: (key, val) => key < x)
            for p in e:
                if p.0 >= x: break
                w.push(p)
        if got != w: bad = "op {k}: drain side {side} {x} {v} gave {got} want {w}"
        for p in w: want[p.0 + 8] = None
    elif op == 6:
        let hi = m.split_at(x)
        let e = entries(want)
        if m.items.collect() != [p for p in e if p.0 < x] or hi.items.collect() != [p for p in e if p.0 >= x]:
            bad = "op {k}: split_at {x}"
        m.join(hi)
    elif op == 7:
        var o: SortedMap<Int, Int> = SortedMap()
        var b: List<?Int> = List.filled(17, None)
        for _ in 0..rng.int(0, 6):
            let j = rng.int(-8, 8)
            let w = rng.int(-9, 9)
            o.set(j, w)
            b[j + 8] = Some(w)
        let join = rng.int(0, 1) == 1
        if join: m.absorb(mut o, join: Some((p, q) => p + q)) else: m.absorb(mut o)
        if not o.is_empty: bad = "op {k}: absorb left other non-empty"
        for j in 0..17:
            if let w = b[j]:
                want[j] = if let p = want[j], join: Some(p + w) else: Some(w)
    else:
        let w = rng.int(0, 2)
        if w == 0:
            m.clear()
            want = List.filled(17, None)
        elif w == 1:
            let t = m.take()
            if not m.is_empty or m.len != 0 or m.first != None: bad = "op {k}: take left entries"
            m = t
        else:
            m.reserve(rng.int(0, 40))
    if bad != "": break
    let e = entries(want)
    let q = rng.int(-9, 9)
    let i = rng.int(-1, e.len + 1)
    let ge = [p for p in e if p.0 >= q]
    let le = [p for p in e if p.0 <= q]
    if m.len != e.len or m.is_empty != (e.len == 0):
        bad = "op {k}: len {m.len} want {e.len}"
    elif m.items.collect() != e or m.each_rev().collect() != rev(e):
        bad = "op {k}: items {m.items.collect()} want {e}"
    elif m.keys.collect() != [p.0 for p in e] or m.values.collect() != [p.1 for p in e]:
        bad = "op {k}: keys or values"
    elif m.first != opt(e, 0) or m.last != last_of(e):
        bad = "op {k}: first or last"
    elif m.get(q) != at(want, q) or m.contains(q) != (at(want, q) != None):
        bad = "op {k}: get {q}"
    elif m.first_ge(q) != opt(ge, 0) or m.first_gt(q) != opt([p for p in e if p.0 > q], 0):
        bad = "op {k}: first_ge or first_gt {q}"
    elif m.last_le(q) != last_of(le) or m.last_lt(q) != last_of([p for p in e if p.0 < q]):
        bad = "op {k}: last_le or last_lt {q}"
    elif m.rank(q) != [p for p in e if p.0 < q].len or m.nth(i) != opt(e, i):
        bad = "op {k}: rank {q} or nth {i}"
    elif m.first_true((key, val) => key >= q) != opt(ge, 0) or m.last_true((key, val) => key <= q) != last_of(le):
        bad = "op {k}: first_true or last_true {q}"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
