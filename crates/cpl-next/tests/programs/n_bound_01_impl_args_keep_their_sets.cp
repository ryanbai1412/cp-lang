// expect: ok, output "3\n1"
// rule: R05.10.2, R08.1.2
// A bound met through an impl takes the type's own fn set: the closures `gen()` yields are
// in the set `cnt`'s `T` and `List(..)`'s element name, though the call adds them later.
fn cnt<S: Seq<T>, T>(s: S) -> Int:
    var n = 0
    for x in s: n += 1
    n
fn gen() -> yields () -> ():
    for i in 0..3: yield () => print(i)
print(cnt(gen()))
let xs = List(gen())
xs[1]()
