// expect: ok, output "ok 600"
// rule: std.heap.MeldHeaps
// A popped element's node goes back to the pool and a later push takes it: heaps that push, pop,
// shift and meld through many reuses keep exactly their own elements, and an offset an element
// carried before its node was freed never reaches the element that reuses the node.
use std.heap.{Meld, MeldHeaps}
var p = MeldHeaps<Int>()
var hs = [Meld<Int>() for _ in 0..4]
var bs: List<List<Int>> = [[] for _ in 0..4]
var checked = 0
for step in 0..600:
    let k = step * 7 % 4
    let x = step * 37 % 101
    if step % 5 == 0:
        p.add_all(hs[k], 3)
        for i in 0..bs[k].len: bs[k][i] += 3
    elif step % 3 == 0:
        bs[k].sort()
        let want = if bs[k].len > 0: Some(bs[k][0]) else: None
        if bs[k].len > 0: bs[k] = bs[k][1..].to_list()
        assert p.pop(mut hs[k]) == want
    elif step % 11 == 1:
        let j = (k + 1) % 4
        hs[k] = p.meld(hs[k], hs[j])
        hs[j] = Meld()
        for i in 0..bs[j].len: bs[k].push(bs[j][i])
        bs[j] = []
    else:
        p.push(mut hs[k], x)
        bs[k].push(x)
    checked += 1
for k in 0..4:
    bs[k].sort()
    assert p.drain(mut hs[k]) == bs[k]
print("ok", checked)
