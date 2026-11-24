// expect: ok, output "ok"
// input: "1 300\n"
// model: Heap
// model: MaxHeap
// model: MinMaxHeap
// model: ErasableHeap
// Heap, MaxHeap, MinMaxHeap and ErasableHeap of Int, each against a sorted List: seeded random
// pushes, pops, push_pop, replace_top, removals, takes and drains, with the contents compared
// after each op. Reads `seed n`; prints the first op that differs.
// std_functions: 8
use std.rand.{Rng}
use std.heap.{Heap, MaxHeap, MinMaxHeap, ErasableHeap}

fn insort(mut a: List<Int>, x: Int):
    var i = 0
    while i < a.len and a[i] < x: i += 1
    a.insert(i, x)

fn rev(a: List<Int>) -> List<Int> = [a[a.len - 1 - i] for i in 0..a.len]

let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var h: Heap<Int> = Heap()
var ha: List<Int> = []
var g: MaxHeap<Int> = MaxHeap()
var hb: Heap<Int> = Heap(by: x => -x)
var ga: List<Int> = []
var m: MinMaxHeap<Int> = MinMaxHeap()
var ma: List<Int> = []
var e: ErasableHeap<Int> = ErasableHeap()
var ea: List<Int> = []
var bad = ""
for k in 0..n:
    let s = rng.int(0, 3)
    let op = rng.int(0, 9)
    let x = rng.int(0, 49)
    if s == 0:
        if op <= 2:
            h.push(x)
            insort(mut ha, x)
        elif op == 3:
            let want = if ha.len == 0: None else: Some(ha.remove(0))
            if h.pop() != want: bad = "op {k}: Heap.pop"
        elif op == 4:
            var want = x
            if ha.len > 0 and ha[0] < x:
                want = ha.remove(0)
                insort(mut ha, x)
            if h.push_pop(x) != want: bad = "op {k}: Heap.push_pop({x})"
        elif op == 5:
            let want = if ha.len == 0: None else: Some(ha.remove(0))
            insort(mut ha, x)
            if h.replace_top(x) != want: bad = "op {k}: Heap.replace_top({x})"
        elif op == 6:
            let b = [rng.int(0, 49) for _ in 0..rng.int(0, 3)]
            h.push_all(b)
            for y in b: insort(mut ha, y)
        elif op == 7:
            let c = rng.int(0, 3)
            var got: List<Int> = []
            if c > 0:
                for y in h.drain():
                    got.push(y)
                    if got.len == c: break
            var want: List<Int> = []
            while want.len < c and ha.len > 0: want.push(ha.remove(0))
            if got != want: bad = "op {k}: Heap.drain {got} want {want}"
        elif op == 8:
            var t = h.take()
            if not h.is_empty or t.each().collect() != ha: bad = "op {k}: Heap.take"
            if x % 2 == 0: h = t
            else: ha = []
        else:
            if x % 3 == 0:
                h.clear()
                ha = []
            else:
                h = Heap.of(rev(ha))
                h.reserve(ha.len + x)
        if bad == "" and (h.len != ha.len or h.is_empty != (ha.len == 0) or h.peek != ha.first or h.each().collect() != ha):
            bad = "op {k}: Heap {h.each().collect()} want {ha}"
    elif s == 1:
        if op <= 2:
            g.push(x)
            hb.push(x)
            insort(mut ga, x)
        elif op == 3:
            let want = if ga.len == 0: None else: Some(ga.remove(ga.len - 1))
            if g.pop() != want or hb.pop() != want: bad = "op {k}: MaxHeap.pop"
        elif op == 4:
            var want = x
            if ga.len > 0 and ga[ga.len - 1] > x:
                want = ga.remove(ga.len - 1)
                insort(mut ga, x)
            if g.push_pop(x) != want or hb.push_pop(x) != want: bad = "op {k}: MaxHeap.push_pop({x})"
        elif op == 5:
            let want = if ga.len == 0: None else: Some(ga.remove(ga.len - 1))
            insort(mut ga, x)
            if g.replace_top(x) != want or hb.replace_top(x) != want: bad = "op {k}: MaxHeap.replace_top({x})"
        elif op == 6:
            let b = [rng.int(0, 49) for _ in 0..rng.int(0, 3)]
            g.push_all(b)
            hb.push_all(b)
            for y in b: insort(mut ga, y)
        elif op == 7:
            let c = rng.int(0, 3)
            var got: List<Int> = []
            if c > 0:
                for y in g.drain():
                    got.push(y)
                    if got.len == c: break
            var want: List<Int> = []
            while want.len < c and ga.len > 0: want.push(ga.pop()!)
            for y in want: let _ = hb.pop()
            if got != want: bad = "op {k}: MaxHeap.drain {got} want {want}"
        elif op == 8:
            var t = g.take()
            if not g.is_empty or t.each().collect() != rev(ga): bad = "op {k}: MaxHeap.take"
            if x % 2 == 0: g = t
            else:
                ga = []
                hb.clear()
        else:
            if x % 3 == 0:
                g.clear()
                hb.clear()
                ga = []
            else:
                g = MaxHeap.of(ga)
                g.reserve(ga.len + x)
                hb = Heap.of(ga, by: x => -x)
        if bad == "" and (g.len != ga.len or g.is_empty != (ga.len == 0) or g.peek != ga.last or g.each().collect() != rev(ga)):
            bad = "op {k}: MaxHeap {g.each().collect()} want {rev(ga)}"
        if bad == "" and (hb.len != ga.len or hb.peek != ga.last or hb.each().collect() != rev(ga)):
            bad = "op {k}: Heap(by:) {hb.each().collect()} want {rev(ga)}"
    elif s == 2:
        if op <= 3:
            m.push(x)
            insort(mut ma, x)
        elif op == 4 or op == 5:
            let want = if ma.len == 0: None else: Some(ma.remove(0))
            if m.pop_first() != want: bad = "op {k}: MinMaxHeap.pop_first"
        elif op == 6 or op == 7:
            let want = if ma.len == 0: None else: Some(ma.remove(ma.len - 1))
            if m.pop_last() != want: bad = "op {k}: MinMaxHeap.pop_last"
        elif op == 8:
            var t = m.take()
            if not m.is_empty or t.each().collect() != ma: bad = "op {k}: MinMaxHeap.take"
            if x % 2 == 0: m = t
            else: ma = []
        else:
            if x % 3 == 0:
                m.clear()
                ma = []
            else:
                m = MinMaxHeap.of(rev(ma))
                m.reserve(ma.len + x)
        if bad == "" and (m.len != ma.len or m.is_empty != (ma.len == 0) or m.first != ma.first or m.last != ma.last or m.each().collect() != ma):
            bad = "op {k}: MinMaxHeap {m.each().collect()} want {ma}"
    else:
        if op <= 3:
            e.push(x)
            insort(mut ea, x)
        elif (op == 4 or op == 5) and ea.len > 0:
            let y = ea.remove(x % ea.len)
            e.remove(y)
        elif op == 6:
            let want = if ea.len == 0: None else: Some(ea.remove(0))
            if e.pop() != want: bad = "op {k}: ErasableHeap.pop"
        elif op == 7:
            var t = e.take()
            if not e.is_empty or t.each().collect() != ea: bad = "op {k}: ErasableHeap.take"
            if x % 2 == 0: e = t
            else: ea = []
        elif op == 8:
            e.clear()
            ea = []
        if bad == "" and (e.len != ea.len or e.is_empty != (ea.len == 0) or e.peek() != ea.first or e.each().collect() != ea):
            bad = "op {k}: ErasableHeap {e.each().collect()} want {ea}"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
