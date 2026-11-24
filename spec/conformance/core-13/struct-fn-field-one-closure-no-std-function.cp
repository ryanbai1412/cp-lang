// expect: ok
// rule: 13.3.2
// tool: build -o -
// lacks: "std::function"
struct Plus(z: Int):
    fn monoid(self) -> Monoid<Int> = Monoid(z, (a, b) => self.op(a, b))
impl Op<Int> for Plus:
    fn op(self, a: Int, b: Int) -> Int = a + b
let k: 1..=8 = read()
let m = Plus(k).monoid()
print(m.op(m.id, 2))
