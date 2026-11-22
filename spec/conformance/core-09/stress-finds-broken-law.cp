// expect: exit 1
// rule: 09.3.9
// tool: stress --seed 1
// has: "comm"
struct Sub: z: Int
impl Op<Int> for Sub:
    fn op(self, a: Int, b: Int) -> Int = a - b
impl Commutative<Int> for Sub
fn total(n: Int) -> Int = [n, 3, 1].fold(Sub(0))
@alt(total)
fn total_slow(n: Int) -> Int = n - 4
let n: 1..=8 = read()
print(total(n))
