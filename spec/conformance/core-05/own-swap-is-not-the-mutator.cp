// expect: ok, output "2 1"
// rule: 05.4.2
// A struct's own non-mut `swap` is not the list mutator, so a `let` receiver may call it.
struct P(a: Int, b: Int):
    fn swap(self) -> P = P(self.b, self.a)
let p = P(1, 2)
let q = p.swap()
print(q.a, q.b)
