// expect: ok, output "2\n4\n2"
// rule: R08.1.10, R05.10.2
// A list literal is a place each element flows into: two different closures are one fn set,
// with or without a type written, and a push adds a third.
let fs = [() => print(1), () => print(2)]
fs[1]()
var gs = [() => print(3)]
gs.push(() => print(4))
gs[1]()
fn cnt<S: Seq<T>, T>(s: S) -> Int:
    var n = 0
    for x in s: n += 1
    n
print(cnt([(x: Int) => x, (x: Int) => x + 1]))
