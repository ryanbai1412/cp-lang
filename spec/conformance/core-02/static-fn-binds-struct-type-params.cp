// expect: ok, output "2 5 8 5"
// rule: 02.6.3
// A static fn's call binds its struct's type parameters from the arguments, so `p` is a `Pair<Int>` and its `yields T` getter walks `Int`s.
struct Pair<T>(a: T, b: T):
    pub fn twin(x: T) -> Pair<T> = Pair(x, x)
    @get pub fn both(self) -> yields T:
        yield self.a
        yield self.b
let p = Pair.twin(4)
let xs = p.both.collect()
var s = 0
for x in p.both: s += x
print(xs.len, xs[0] + 1, s, p.a + 1)
