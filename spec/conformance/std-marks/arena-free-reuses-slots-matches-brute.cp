// expect: ok, output "ok"
// A pool with frees and reuse against a map of live handles: `len`, `items` (freed slots
// skipped, by handle) and reuse of the last freed slot first.
use std.marks.{Arena, Id}

var ok = true
for round in 0..30:
    var a = Arena<Int>(cap: rand(0, 8))
    var live: List<(Int, Int)> = []
    var freed: List<Int> = []
    var slots = 0
    for step in 0..60:
        if live.len > 0 and rand(0, 2) == 0:
            let k = rand(0, live.len - 1)
            let (h, _) = live[k]
            a.free(Id<Int>(h))
            freed.push(h)
            live = [live[j] for j in 0..live.len if j != k]
        else:
            let v = round * 1000 + step
            let id = a.alloc(v)
            let want = if let f = freed.pop(): f else: slots
            if want == slots: slots += 1
            ok = ok and id.raw == want
            live.push((id.raw, v))
        for (h, v) in live: ok = ok and a.get(Id<Int>(h)) == v
        let seen = [(id.raw, x) for (id, x) in a.items]
        ok = ok and seen == live.sorted() and a.len == live.len and a.is_empty == (live.len == 0)
    a.clear()
    ok = ok and a.is_empty and a.alloc(1).raw == 0
print(if ok: "ok" else: "mismatch")
