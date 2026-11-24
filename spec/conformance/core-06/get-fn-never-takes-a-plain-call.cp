// expect: ok, output "2 1 4 3\n6 5\n3"
// rule: 06.4.2
// A `@get` fn never shadows the prelude fn of its name in a plain call, and a generic one binds from the receiver.
@get fn swap<A>(self: (A, A)) -> (A, A):
    let (a, b) = self
    (b, a)
@get fn fst<A>(self: (A, A)) -> A:
    let (a, _) = self
    a
var x = [1]
var y = [2]
swap(mut x, mut y)
print(x[0], y[0], (3, 4).swap)
let p = (5, 6)
print(p.swap)
print((3, 4).fst)
