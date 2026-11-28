// expect: E0602
// rule: R05.10.3
// R05.10.3: calling a nested fn that writes its captured `xs` writes `xs`, so a view of it still used after the call is E0602.
fn go() -> Int:
    var xs = [1, 2, 3]
    let v = xs.rev
    fn put(i: Int):
        xs[i] = i
    put(0)  // <- here
    v.len
print(go())
