// A by-value param nothing reads after its sink moves into it, even with a
// closure in the body; one a closure made earlier holds is copied.
fn join(a: List<(Int, Int)>, b: List<(Int, Int)>) -> List<(Int, Int)>:
    var all = a
    for x in b:
        if let i = (0..all.len).find(i => all[i].0 == x.0): all[i].1 += x.1
        else: all.push(x)
    all
fn held(a: List<Int>) -> Int:
    let g = () => a.len
    var all = a
    all.push(1)
    g() * 10 + all.len
print(join([(1, 2), (3, 4)], [(3, 5), (6, 7)]), held([1, 2]))
