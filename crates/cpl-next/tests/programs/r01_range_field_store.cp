// expect: R0201
// A field of a range type: a read is an `Int`, a store is checked, `+=` included.
struct P:
    var a: 0..=10
var p = P(3)
p.a = p.a + 4
print(p.a)
p.a += 9 // <- here
print(p.a)
