// expect: ok, output "ok"
// input: "1 300\n"
// model: TimeTree
// model: cdq
// model: offline_over
// Each op is one seeded random timeline: TimeTree<Int> with random add_interval and add_range
// events, walked by steps, walk and run (and offline_over) on a stack of applied events, against
// the plain list of events whose interval covers each time. cdq on a random list must show each
// pair i < j exactly once, left before right. Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
use std.timetree.{TimeTree, cdq, offline_over}
let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var bad = ""
for k in 0..n:
    let q = rng.int(0, 9)
    var tt = TimeTree<Int>(q)
    var spans: List<(Int, Int)> = []
    for i in 0..rng.int(0, 6):
        let lo = rng.int(0, q)
        let hi = rng.int(0, q)
        if rng.int(0, 1) == 0: tt.add_interval(lo, hi, i)
        else: tt.add_range(lo..max(lo, hi), i)
        spans.push((lo, max(lo, hi)))
    let want = [sorted([i for i in 0..spans.len if spans[i].0 <= t < spans[i].1]) for t in 0..q]
    var live: List<Int> = []
    var seen: List<List<Int>> = []
    var applied: List<Int> = []
    for step in tt.steps():
        match step:
            Apply(id):
                live.push(tt.event(id))
                applied.push(id)
            Query(t):
                if t != seen.len: bad = "op {k}: query {t} out of order"
                seen.push(sorted(live))
            Undo:
                if live.len == 0: bad = "op {k}: undo with nothing applied"
                else: live.pop()
    if bad == "" and (seen != want or live.len != 0):
        bad = "op {k}: steps answer {seen} want {want}"
    elif bad == "" and tt.walk().collect() != tt.steps():
        bad = "op {k}: walk differs from steps"
    var stack: List<Int> = []
    let ran = tt.run(
        mark: () => stack.len,
        apply: e => stack.push(e),
        answer: _ => sorted(stack),
        undo: m => do:
            while stack.len > m: stack.pop()
    )
    let over = offline_over(
        q,
        [i for i in 0..spans.len],
        by: i => spans[i],
        mark: () => stack.len,
        apply: e => stack.push(e),
        answer: _ => sorted(stack),
        undo: m => do:
            while stack.len > m: stack.pop()
    )
    if bad == "" and ran != want: bad = "op {k}: run answer {ran} want {want}"
    elif bad == "" and over != want: bad = "op {k}: offline_over answer {over} want {want}"
    let m = rng.int(0, 9)
    var pairs = List.filled(m * m, 0)
    var order_ok = true
    cdq([i for i in 0..m], on_cross: (l, r) => do:
        for i in l:
            for j in r:
                pairs[i * m + j] += 1
                if i >= j: order_ok = false
    )
    for i in 0..m:
        for j in 0..m:
            if pairs[i * m + j] != (if i < j: 1 else: 0): order_ok = false
    if bad == "" and not order_ok: bad = "op {k}: cdq over {m} events"
    if bad != "": break
print(if bad == "": "ok" else: bad)
