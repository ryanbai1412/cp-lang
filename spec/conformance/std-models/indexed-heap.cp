// expect: ok, output "ok"
// input: "1 300\n"
// model: IndexedHeap
// model: Settle
// IndexedHeap<V, Int> against a List<?Int> of keys, and Settle<Int, Int> (best min and max)
// against key and done lists: seeded random sets, decreases, increases, removals, offers and
// pops, with every key compared after each op. Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
use std.heap.{IndexedHeap, Settle}

domain V = 0..8

fn least(keys: List<?Int>) -> ?(V, Int):
    var best: ?(V, Int) = None
    for i in 0..keys.len:
        if let k = keys[i]:
            if best == None or k < best!.1: best = Some((V(i), k))
    best

let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var h: IndexedHeap<V, Int> = IndexedHeap()
var keys: List<?Int> = [None for _ in 0..8]
var lo: Settle<Int, Int> = Settle()
var hi: Settle<Int, Int> = Settle(best: max)
var cur: List<List<?Int>> = [[None for _ in 0..6], [None for _ in 0..6]]
var done: List<List<Bool>> = [[false for _ in 0..6], [false for _ in 0..6]]
var bad = ""
for k in 0..n:
    let op = rng.int(0, 9)
    let i = rng.int(0, 7)
    let x = rng.int(0, 20)
    let v = V(i)
    if op <= 1:
        h.set(v, x)
        keys[i] = Some(x)
    elif op == 2:
        let want = keys[i] == None or x < keys[i]!
        if want: keys[i] = Some(x)
        if h.decrease(v, x) != want: bad = "op {k}: decrease({i}, {x})"
    elif op == 3:
        let want = keys[i] == None or x > keys[i]!
        if want: keys[i] = Some(x)
        if h.increase(v, x) != want: bad = "op {k}: increase({i}, {x})"
    elif op == 4:
        let want = keys[i]
        keys[i] = None
        if h.remove(v) != want: bad = "op {k}: remove({i})"
    elif op == 5:
        let want = least(keys)
        if let (w, _) = want: keys[w.ord] = None
        if h.pop() != want: bad = "op {k}: pop"
    elif op == 6 and x % 4 == 0:
        h.clear()
        keys = [None for _ in 0..8]
    elif op == 7 or op == 8:
        let s = x % 2
        let key = i % 6
        if not done[s][key]:
            if let old = cur[s][key]:
                cur[s][key] = Some(if s == 0: min(old, x) else: max(old, x))
            else:
                cur[s][key] = Some(x)
        if s == 0: lo.offer(key, x)
        else: hi.offer(key, x)
    elif op == 9:
        let s = x % 2
        var want: ?Int = None
        for j in 0..6:
            if not done[s][j]:
                if let p = cur[s][j]:
                    if want == None or (s == 0 and p < want!) or (s == 1 and p > want!): want = Some(p)
        let got = if s == 0: lo.pop() else: hi.pop()
        if let (j, p) = got:
            if want != Some(p) or done[s][j] or cur[s][j] != Some(p): bad = "op {k}: Settle.pop"
            else: done[s][j] = true
        elif want != None: bad = "op {k}: Settle.pop None"
    if bad == "":
        var want: List<(V, Int)> = []
        var rest = keys.clone()
        while let e = least(rest):
            want.push(e)
            rest[e.0.ord] = None
        let cnt = [y for y in keys if y != None].len
        if h.len != cnt or h.is_empty != (cnt == 0) or h.peek != least(keys) or h.each().collect() != want:
            bad = "op {k}: IndexedHeap {h.each().collect()} want {want}"
        for j in 0..8:
            if h.contains(V(j)) != (keys[j] != None) or h.get(V(j)) != keys[j]: bad = "op {k}: IndexedHeap at {j}"
        for j in 0..6:
            if lo.settled(j) != done[0][j] or lo.get(j) != cur[0][j]: bad = "op {k}: Settle(min) at {j}"
            if hi.settled(j) != done[1][j] or hi.get(j) != cur[1][j]: bad = "op {k}: Settle(max) at {j}"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
