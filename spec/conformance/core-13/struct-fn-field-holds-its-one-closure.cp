// expect: ok, output "8\n5\n15\n13"
// rule: 13.3.2
// input: "3"
// std_functions: 1
// A struct fn field only ever built from one capturing closure holds that closure as its own
// type; a field two closures reach holds either. Both call through the field the same way.
struct Plus(z: Int):
    fn monoid(self) -> Monoid<Int> = Monoid(z, (a, b) => self.op(a, b) + self.z)
impl Op<Int> for Plus:
    fn op(self, a: Int, b: Int) -> Int = a + b
struct Op2(f: (Int, Int) -> Int)
let k: 1..=8 = read()
let m = Plus(k).monoid()
print(m.op(m.id, 2))
print(m.op(0, 2))
fn run(k: Int):
    let ops = [Op2((a, b) => a + b + k * 2 + 4), Op2((a, b) => a * b + k + 4)]
    for o in ops: print(o.f(2, 3))
run(k)
