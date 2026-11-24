use std.seq.{compress, MonoStack, OnlineLis}
let n: Int = read()
let xs = [i * 7919 % n for i in 0..n]
let ys = [i for i in 0..n]
let (domain C, cx) = compress(xs)
let rg = ys.ring
var s = 0
var ms = MonoStack<Int, Int>()
var lis = OnlineLis<Int>()
for rep in 0..4:
    ms = MonoStack<Int, Int>()
    lis = OnlineLis<Int>()
    for i in 0..n:
        s += ms.push(i, xs[i]) ?? 0 // timed: seq.MonoStack.push
        s += lis.push(xs[i]) // timed: seq.OnlineLis.push
var full = OnlineLis<Int>()
var up = MonoStack<Int, Int>()
for y in ys:
    s += full.push(y)
    s += up.push(y, n - y) ?? 0
for r in 0..262144:
    let k = r % n
    s += ms.len // timed: seq.MonoStack.len
    s += lis.len // timed: seq.OnlineLis.len
    s += Int(ms.is_empty) // timed: seq.MonoStack.is_empty
    s += Int(lis.is_empty) // timed: seq.OnlineLis.is_empty
    s += (ms.top ?? (index: 0, val: 0)).val // timed: seq.MonoStack.top
    let m = ms.mark() // timed: seq.MonoStack.mark
    s += ms.push(n + r, n + r) ?? 0
    ms.undo_to(m) // timed: seq.MonoStack.undo_to
    let j = lis.mark() // timed: seq.OnlineLis.mark
    s += lis.push(n + r)
    lis.undo_to(j) // timed: seq.OnlineLis.undo_to
    var tm = MonoStack<Int, Int>()
    s += tm.push(0, r) ?? 0
    tm.clear() // timed: seq.MonoStack.clear
    var tl = OnlineLis<Int>()
    s += tl.push(r)
    tl.clear() // timed: seq.OnlineLis.clear
    s += cx.get(C.nth(k)) // timed: seq.Compressed.get
    s += Int(cx.index_of(xs[k]) == C.nth(0)) // timed: seq.Compressed.index_of
    s += Int(cx.try_index_of(k + n) == None) // timed: seq.Compressed.try_index_of
    s += Int(cx.contains(k)) // timed: seq.Compressed.contains
    s += Int(cx.first_ge(k) == None) // timed: seq.Compressed.first_ge
    s += Int(cx.first_gt(k) == None) // timed: seq.Compressed.first_gt
    s += Int(cx.last_le(k) == None) // timed: seq.Compressed.last_le
    s += Int(cx.last_lt(k) == None) // timed: seq.Compressed.last_lt
    s += cx.len // timed: seq.Compressed.len
    s += ys.ring.next(k) // timed: seq.ring
    s += rg.step(k, 5) // timed: seq.Ring.step
    s += rg.next(k) // timed: seq.Ring.next
    s += rg.prev(k) // timed: seq.Ring.prev
    s += Int(cx.is_empty) // timed: seq.Compressed.is_empty
    s += Int(rg.span(k, 5).is_empty) // timed: seq.RingSlice.is_empty
    s += rg.span(k, 5).len // timed: seq.Ring.span
for r in 0..256:
    s += cx.slice(C.all).len // timed: seq.Compressed.slice
    s += compress(xs).1.len // timed: seq.compress
for r in 0..512:
    s += cx.items.count(p => p.1 >= 0) // timed: seq.Compressed.items
    s += up.items.count(p => p.1 >= 0) // timed: seq.MonoStack.items
for r in 0..128:
    s += full.items.count() // timed: seq.OnlineLis.items
    s += rg.span(r, n / 2).each().count() // timed: seq.RingSlice.each
    s += full.reconstruct().len // timed: seq.OnlineLis.reconstruct
for r in 0..32:
    for i in rg.from(r): // timed: seq.Ring.from
        s += i // timed: seq.Ring.from
    for w in rg.windows(5): // timed: seq.Ring.windows
        s += w.start // timed: seq.Ring.windows
    for w in rg.windows(5):
        s += w.len // timed: seq.RingSlice.len
        s += w.start // timed: seq.RingSlice.start
        s += w.end // timed: seq.RingSlice.end
        s += w.range.len // timed: seq.RingSlice.range
        for d in w.keys: // timed: seq.RingSlice.keys
            s += d // timed: seq.RingSlice.keys
print(s)
