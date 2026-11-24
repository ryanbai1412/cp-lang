// expect: ok, output "3 10 20\n3 10 20"
// rule: 05.4.3
// A closure passed beside a `mut` argument reads the caller's place, which holds the callee's value
// while the call runs: a map's `len` read inside `update` counts the keys it holds so far.
var hs: Map<Int, Int> = Map()
for i in 0..4:
    hs.update(i % 3, -1, v => if v < 0: hs.len * 10 else: v + hs.len)
print(hs[0], hs[1], hs[2])
fn local() -> (Int, Int, Int):
    var m: Map<Int, Int> = Map()
    for i in 0..4:
        m.update(i % 3, -1, v => if v < 0: m.len * 10 else: v + m.len)
    (m[0], m[1], m[2])
let (a, b, c) = local()
print(a, b, c)
