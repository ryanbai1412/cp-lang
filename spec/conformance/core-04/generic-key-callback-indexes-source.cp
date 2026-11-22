// expect: ok, output "12 12"
// rule: 04b.1.5
fn total<E, T>(xs: [E]T, w: E -> Int) -> Int:
    var s = 0
    for e in E: s += w(e)
    s
let counted: [0..2]Int = [5, 7]
domain M = 0..2
let keyed: [M]Int = [5, 7]
print(total(counted, w: e => counted[e]), total(keyed, w: e => keyed[e]))
