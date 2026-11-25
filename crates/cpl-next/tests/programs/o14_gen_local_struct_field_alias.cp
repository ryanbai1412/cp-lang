// expect: E0602
// Walk a field of a local struct while the body replaces the whole struct.
struct W:
    a: List<Int>
fn go() -> Int:
    var w = W([1, 2, 3])
    var s = 0
    for x in w.a:
        w = W([9])  // <- here
        s += x
    s
print(go())
