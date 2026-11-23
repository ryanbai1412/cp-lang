// IOI 2011 Rice Hub, as it builds today: the window is two hand-moved
// positions, and the sorted input needs its own generator (R3, R5).
// original: corpus/submissions/3732163.cpp
let n: 1..=1e5 = read()
let len: 1..=1e9 = read()
let budget: 0..=2e15 = read()
domain Field = 1..=n

fn sorted_fields() -> [Field]1..=1e9:
    var at: [Field]1..=1e9 = 1
    for i in Field: at[i] = rand(1, len)
    at.sort()
    at

let x: [Field]1..=1e9 = read(gen: sorted_fields)
let pre = prefix(x)

// the hub goes on the middle field of the block
fn cost(l: Field, r: Field) -> Int:
    let mid = Field((l.label + r.label) / 2)
    (pre.fold(mid..=r) - pre.fold(l..=mid)) - x[mid] * ((r - mid) - (mid - l))

var best = 0
var l = Field.first!
for r in Field:
    while cost(l, r) > budget: l = l.next!
    best max= r - l + 1
print(best)
