// expect: ok, output "ok"
// input: "1 300\n"
// model: Counter
// model: push
// model: pop
// Counter<Int, Int> against a List<Int> of counts plus the initial value: seeded random add,
// set, remove, push, pop, clear, reserve, take, absorb and rebuilds by new (with and without a
// named init) and of, with get, contains, len, items, keys, values, total and most_common
// compared after each op. Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}

fn stored(a: List<Int>, init: Int) -> List<(Int, Int)> = [(i, a[i]) for i in 0..a.len if a[i] != init]

let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var init = 0
var c: Counter<Int, Int> = Counter()
var a = List.filled(12, 0)
var bad = ""
for k in 0..n:
    let op = rng.int(0, 10)
    let i = rng.int(0, 11)
    let v = rng.int(-3, 3)
    if op <= 1:
        c.add(i, v)
        a[i] += v
    elif op == 2:
        c.set(i, v)
        a[i] = v
    elif op == 3:
        let want = if a[i] == init: None else: Some(a[i])
        if c.remove(i) != want: bad = "op {k}: remove {i}"
        a[i] = init
    elif op == 4:
        c.push(i)
        a[i] += 1
    elif op == 5:
        c.pop(i)
        a[i] -= 1
    elif op == 6:
        if rng.int(0, 1) == 0:
            c.clear()
            a = List.filled(12, init)
        else:
            c.reserve(rng.int(-1, 50))
    elif op == 7:
        let t = c.take()
        if not c.is_empty or c.get(i) != init: bad = "op {k}: take left entries"
        c = t
    elif op == 8:
        var o: Counter<Int, Int> = Counter(init:)
        var b = List.filled(12, init)
        for _ in 0..rng.int(0, 6):
            let j = rng.int(0, 11)
            let w = rng.int(-3, 3)
            o.add(j, w)
            b[j] += w
        c.absorb(mut o)
        if not o.is_empty: bad = "op {k}: absorb left other non-empty"
        for j in 0..12:
            if b[j] != init: a[j] = if a[j] != init: a[j] + b[j] else: b[j]
    elif op == 9:
        let xs = [rng.int(0, 11) for _ in 0..rng.int(0, 10)]
        init = rng.int(-1, 1)
        c = Counter(xs, init:)
        a = List.filled(12, init)
        for x in xs: a[x] += 1
    else:
        let xs = [rng.int(0, 11) for _ in 0..rng.int(0, 10)]
        init = 0
        c = Counter.of(xs)
        a = List.filled(12, 0)
        for x in xs: a[x] += 1
    if bad != "": break
    let want = stored(a, init)
    let top = rng.int(0, 13)
    let mc = want.sorted(by: e => (-e.1, e.0))
    if c.len != want.len or c.is_empty != (want.len == 0):
        bad = "op {k}: len {c.len} want {want.len}"
    elif c.items.sorted() != want:
        bad = "op {k}: items {c.items.sorted()} want {want}"
    elif c.keys.sorted() != [e.0 for e in want] or c.values.sorted() != [e.1 for e in want].sorted():
        bad = "op {k}: keys or values"
    elif c.get(i) != a[i] or c[i] != a[i] or c.contains(i) != (a[i] != init) or c.get(99) != init:
        bad = "op {k}: get {i}"
    elif c.total() != sum(e.1 for e in want):
        bad = "op {k}: total {c.total()}"
    elif c.most_common(top) != [mc[j] for j in 0..min(top, mc.len)]:
        bad = "op {k}: most_common {top} = {c.most_common(top)}"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
