// expect: E0603
// rule: R05.10.2, R8
// A struct holding a by-reference closure is lent and may not be returned (R05.10.2, R8).
struct Counter:
    next: () -> Int
fn make() -> Counter:
    var c = 0
    Counter(() => { c += 1; c })  // <- here
let k = make()
print(k.next(), k.next())
