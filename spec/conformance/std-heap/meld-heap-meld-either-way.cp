// expect: ok, output "ok 200"
// rule: std.heap.MeldHeap.meld
// `meld` copies the smaller heap's nodes into the larger, in whichever direction it is called; the
// result, pending `add_all` offsets included, matches a sorted list either way.
use std.heap.{MeldHeap}
domain H = 0..6
var hs: [H]MeldHeap<Int> = [H] _ => MeldHeap()
var want: [H]List<Int> = [H] _ => []
var ok = 0
for _ in 0..200:
    let i = H(rand(0, 5))
    let j = H(rand(0, 5))
    match rand(0, 3):
        0 =>
            let x = rand(-50, 50)
            hs[i].push(x)
            want[i].push(x)
        1 =>
            let d = rand(-5, 5)
            hs[i].add_all(d)
            want[i] = want[i].map(x => x + d).collect()
        2 =>
            if i != j:
                hs[i].meld(hs[j].take())
                want[i] = want[i] + want[j]
                want[j] = []
        _ =>
            let got = hs[i].pop()
            want[i].sort()
            let exp = if want[i].len > 0: Some(want[i][0]) else: None
            if exp != None: want[i] = want[i][1..].collect()
            if got != exp: ok -= 1000
    var w = want[i]
    w.sort()
    var h = hs[i]
    if h.drain() == w and hs[i].len == w.len: ok += 1
print("ok", ok)
