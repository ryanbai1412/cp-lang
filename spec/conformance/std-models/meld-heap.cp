// expect: ok, output "ok"
// input: "1 300\n"
// model: MeldHeap
// model: MeldHeaps
// model: Meld
// Two MeldHeap<Int> and three Meld<Int> handles in one MeldHeaps<Int> pool, each against a
// sorted List: seeded random pushes, pops, melds, lazy adds, takes and drains, with the
// contents compared after each op. Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
use std.heap.{MeldHeap, MeldHeaps, Meld}

fn insort(mut a: List<Int>, x: Int):
    var i = 0
    while i < a.len and a[i] < x: i += 1
    a.insert(i, x)

fn merged(a: List<Int>, b: List<Int>) -> List<Int>:
    var out = a.clone()
    for y in b: insort(mut out, y)
    out

let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var hs: List<MeldHeap<Int>> = [MeldHeap(), MeldHeap()]
var ha: List<List<Int>> = [[], []]
var p = MeldHeaps<Int>()
var ms: List<Meld<Int>> = [p.of([]), Meld<Int>(), p.of([])]
var ma: List<List<Int>> = [[], [], []]
var bad = ""
for k in 0..n:
    let op = rng.int(0, 19)
    let x = rng.int(0, 49)
    let i = rng.int(0, 1)
    let j = rng.int(0, 2)
    if op <= 2:
        hs[i].push(x)
        insort(mut ha[i], x)
    elif op == 3:
        let b = [rng.int(0, 49) for _ in 0..rng.int(0, 3)]
        hs[i].push_all(b)
        ha[i] = merged(ha[i], b)
    elif op == 4:
        let want = if ha[i].len == 0: None else: Some(ha[i].remove(0))
        if hs[i].pop() != want: bad = "op {k}: MeldHeap.pop"
    elif op == 5:
        if x % 2 == 0:
            var o = hs[1 - i].take()
            hs[i].absorb(mut o)
            if not o.is_empty: bad = "op {k}: MeldHeap.absorb left other non-empty"
        else:
            hs[i].meld(hs[1 - i].take())
        ha[i] = merged(ha[i], ha[1 - i])
        ha[1 - i] = []
    elif op == 6:
        let b = [rng.int(0, 49) for _ in 0..rng.int(0, 4)]
        hs[i].meld(MeldHeap.of(b))
        ha[i] = merged(ha[i], b)
    elif op == 7:
        let d = rng.int(-20, 20)
        hs[i].add_all(d)
        ha[i] = [y + d for y in ha[i]]
    elif op == 8:
        if x % 3 == 0:
            if hs[i].drain() != ha[i]: bad = "op {k}: MeldHeap.drain"
            ha[i] = []
        elif x % 3 == 1:
            hs[i].clear()
            ha[i] = []
        else:
            var t = hs[i].take()
            if not hs[i].is_empty or t.each().collect() != ha[i]: bad = "op {k}: MeldHeap.take"
            hs[i] = t
    elif op <= 11:
        var h = ms[j]
        p.push(mut h, x)
        ms[j] = h
        insort(mut ma[j], x)
    elif op == 12:
        let b = [rng.int(0, 49) for _ in 0..rng.int(0, 3)]
        var h = ms[j]
        p.push_all(mut h, b)
        ms[j] = h
        ma[j] = merged(ma[j], b)
    elif op == 13 or op == 14:
        var h = ms[j]
        let want = if ma[j].len == 0: None else: Some(ma[j].remove(0))
        if p.pop(mut h) != want: bad = "op {k}: MeldHeaps.pop"
        ms[j] = h
    elif op == 15:
        let o = (j + 1 + x % 2) % 3
        ms[j] = p.meld(ms[j], ms[o])
        ms[o] = Meld<Int>()
        ma[j] = merged(ma[j], ma[o])
        ma[o] = []
    elif op == 16:
        let d = rng.int(-20, 20)
        p.add_all(ms[j], d)
        ma[j] = [y + d for y in ma[j]]
    elif op == 17:
        var h = ms[j]
        if p.drain(mut h) != ma[j] or not h.is_empty: bad = "op {k}: MeldHeaps.drain"
        ms[j] = if x % 2 == 0: h else: p.of([y for y in ma[j]])
        if x % 2 == 0: ma[j] = []
    elif op == 18:
        p.reserve(x)
        let b = [rng.int(0, 49) for _ in 0..rng.int(0, 4)]
        ms[j] = p.of(b)
        ma[j] = merged([], b)
    for t in 0..2:
        let h = hs[t]
        let a = ha[t]
        if bad == "" and (h.len != a.len or h.is_empty != (a.len == 0) or h.peek != a.first or h.top != a.first or h.each().collect() != a):
            bad = "op {k}: MeldHeap {t} {h.each().collect()} want {a}"
    for t in 0..3:
        if bad == "" and (ms[t].len != ma[t].len or ms[t].is_empty != (ma[t].len == 0) or p.top(ms[t]) != ma[t].first):
            bad = "op {k}: Meld {t} len {ms[t].len} want {ma[t]}"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
