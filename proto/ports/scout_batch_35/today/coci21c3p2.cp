// Cijanobakterije, in the forms proto/cpc builds today. See ../coci21c3p2.cp.
// original: corpus/submissions/4196157.cpp
let n: Int = read()
let m: Int = read()
domain V = 1..=n
var g: [V]List<V> = []
for _ in 0..m:
    let u: V = read()
    let v: V = read()
    g[u].push(v)
    g[v].push(u)

var seen: [V]Bool = false
var longest = 0
fn down(u: V, parent: ?V) -> Int:
    seen[u] = true
    var deep = 0
    for v in g[u]:
        if parent == none or v != parent!:
            let d = down(v, u)
            longest max= deep + d + 1
            deep max= d
    deep + 1

var total = 0
for root in V:
    if not seen[root]:
        longest = 1
        down(root, none)
        total += longest
print(total)
