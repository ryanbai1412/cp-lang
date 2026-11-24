// expect: ok, output "5 2"
// rule: 08.1.11
fn mx(a: Int, b: Int) -> Int = if a > b: a else: b
impl Op<Int> for type_of(mx):
    fn op(self, a: Int, b: Int) -> Int = mx(a, b)
fn both<M: Op<Int>>(m: M, a: Int, b: Int) -> Int = m.op(a, b)
fn apply(f: (Int, Int) -> Int, a: Int, b: Int) -> Int = f(a, b)
print(both(mx, 2, 5), apply((a: Int, b: Int) => a - b, 5, 3))
