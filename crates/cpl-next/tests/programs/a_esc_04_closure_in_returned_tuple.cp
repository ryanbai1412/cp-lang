// expect: E0603
// rule: R05.7.8
// A lent closure returned inside a tuple.
fn make() -> (Int, () -> Int):
    var c = 0
    let f = () => { c += 1; c }
    (1, f)  // <- here
let (a, g) = make()
print(a, g())
