// expect: ok, output "1\n0\n2\n4"
// rule: 01.7.1
// An `if` whose condition binds names gives one type: the `else`'s `[]` is a list of what the binding branch gives.
enum E:
    A(x: Int)
    B
let o: ?Int = Some(2)
print((if o is Some(x): [x] else: []).len)
let p: ?Int = None
print((if p is Some(x): [x] else: []).len)
let f = (c: E) => if c is E.A(x): [x, x] else: []
print(f(E.A(3)).len)
let es = [E.A(3), E.B, E.A(4)]
print(List(es.flat_map(c => if c is E.A(x): [x, x] else: [])).len)
