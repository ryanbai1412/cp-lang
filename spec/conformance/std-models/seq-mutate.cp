// expect: ok, output "ok"
// input: "1 300\n"
// model: pop_if
// model: pop_while
// model: take_all
// model: clone
// model: get_or_insert
// model: update
// model: union
// model: intersect
// model: subtract
// model: next_permutation
// model: prev_permutation
use std.rand.{Rng}

fn lex_less(a: List<Int>, b: List<Int>) -> Bool:
    for i in 0..a.len:
        if a[i] != b[i]: return a[i] < b[i]
    false

fn as_set(b: List<Bool>) -> List<Int> = [i for i in 0..b.len if b[i]]

let seed = read<Int>()
let n = read<Int>()
var rng = Rng(seed)
var a: List<Int> = []
var m: Map<Int, Int> = Map()
var mb = [-1 for _ in 0..8]
var s: Set<Int> = Set()
var sb = [false for _ in 0..8]
var bad = ""
for op in 0..n:
    let k = rng.int(0, 13 - 1)
    let x = rng.int(0, 10 - 1) - 3
    var why = ""
    if k == 0 and a.len < 12:
        let i = rng.int(0, a.len + 1 - 1)
        let want = [a[j] for j in 0..i] + [x] + [a[j] for j in i..a.len]
        a.insert(i, x)
        if a != want: why = "insert {i} {x}"
    elif k == 1 and a.len > 0:
        let i = rng.int(0, a.len - 1)
        let want = [a[j] for j in 0..a.len if j != i]
        let a_before_i = a[i]
        let got = a.remove(i)
        if a != want or got != a_before_i: why = "remove {i}"
    elif k == 2:
        let want = if a.len > 0 and a[a.len - 1] > x: Some(a[a.len - 1]) else: None
        let rest = if want != None: [a[j] for j in 0..a.len - 1] else: a.clone()
        if a.pop_if(v => v > x) != want or a != rest: why = "pop_if > {x}"
    elif k == 3:
        var t = a.len
        while t > 0 and a[t - 1] < x: t -= 1
        let want = [a[j] for j in (t..a.len).rev]
        let rest = [a[j] for j in 0..t]
        if a.pop_while(v => v < x) != want or a != rest: why = "pop_while < {x}"
    elif k == 4:
        var c = a.clone()
        c.push(x)
        if c.len != a.len + 1 or [c[j] for j in 0..a.len] != a: why = "clone"
        elif rng.int(0, 4 - 1) == 0:
            let want = [v for v in a]
            if a.take_all() != want or a.len != 0: why = "take_all"
    elif k == 5:
        let key = rng.int(0, 8 - 1)
        let want = if mb[key] == -1: x + 3 else: mb[key]
        if mb[key] == -1: mb[key] = x + 3
        if m.get_or_insert(key, x + 3) != want: why = "get_or_insert {key}"
    elif k == 6:
        let key = rng.int(0, 8 - 1)
        mb[key] = (if mb[key] == -1: 2 else: mb[key]) * 3 % 11 + x + 3
        m.update(key, 2, v => v * 3 % 11 + x + 3)
    elif k >= 7 and k <= 9:
        var t: Set<Int> = Set()
        var tb = [false for _ in 0..8]
        for _ in 0..rng.int(0, 5 - 1):
            let v = rng.int(0, 8 - 1)
            t.insert(v)
            tb[v] = true
        if k == 7:
            s.union(t)
            sb = [sb[i] or tb[i] for i in 0..8]
        elif k == 8:
            s.intersect(t)
            sb = [sb[i] and tb[i] for i in 0..8]
        else:
            s.subtract(t)
            sb = [sb[i] and not tb[i] for i in 0..8]
        if s.sorted() != as_set(sb): why = "set op {k} with {as_set(tb)}"
    elif k == 10:
        let v = rng.int(0, 4 - 1)
        s.insert(v)
        sb[v] = true
    elif k >= 11:
        var p = [rng.int(0, 3 - 1) for _ in 0..rng.int(0, 5 - 1)]
        let before = p.clone()
        let fwd = k == 11
        var best: ?List<Int> = None
        var q = p.sorted()
        var all: List<List<Int>> = []
        // brute: every arrangement by repeated adjacent search over sorted order
        var seen = [q.clone()]
        var grow = true
        while grow:
            grow = false
            for w in seen.clone():
                for i in 0..w.len:
                    for j in 0..w.len:
                        var z = w.clone()
                        let tmp = z[i]
                        z[i] = z[j]
                        z[j] = tmp
                        if not seen.any(y => y == z):
                            seen.push(z)
                            grow = true
        for w in seen:
            if (fwd and lex_less(before, w)) or (not fwd and lex_less(w, before)):
                if best == None or (fwd and lex_less(w, best!)) or (not fwd and lex_less(best!, w)):
                    best = Some(w)
        let moved = if fwd: p.next_permutation() else: p.prev_permutation()
        let want = best ?? (if fwd: before.sorted() else: before.sorted().rev.collect())
        if moved != (best != None) or p != want: why = "{if fwd: "next" else: "prev"}_permutation {before}"
    if why == "":
        for key in 0..8:
            if m.get(key) != (if mb[key] == -1: None else: Some(mb[key])): why = "map at {key}"
    if why != "":
        bad = "op {op}: {why}"
        break
print(if bad == "": "ok" else: bad)
