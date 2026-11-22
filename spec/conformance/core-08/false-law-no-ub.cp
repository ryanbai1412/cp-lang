// expect: ok, output "done"
// rule: 08.3.4 09.5.3
struct Sub: z: Int
impl Op<Int> for Sub:
    fn op(self, a: Int, b: Int) -> Int = a - b
impl Commutative<Int> for Sub
let r = [5, 3, 1].fold(Sub(0))
print("done")
