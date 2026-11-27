// expect: ok, output "15 abab"
// rule: R05.2.2
// A field whose type is a type parameter reads as a copy in each instance, so a result built
// from it does not borrow the struct it was read from.
struct Pair<T>(a: T, op: (T, T) -> T)
fn fold<T>(x: T, k: Int, p: Pair<T>) -> T:
    let m = Pair(p.a, p.op)
    var acc = m.a
    for i in 0..k: acc = m.op(acc, x)
    acc
print(fold(5, 3, Pair(0, (a, b) => a + b)), fold("ab", 2, Pair("", (a, b) => a + b)))
