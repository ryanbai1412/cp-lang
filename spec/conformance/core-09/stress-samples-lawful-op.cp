// expect: ok
// rule: 09.3.9
// tool: stress --seed 1
struct Plus: z: Int
impl Op<Int> for Plus:
    fn op(self, a: Int, b: Int) -> Int = a + b
impl Commutative<Int> for Plus
fn total(n: Int) -> Int = [n, 2, 3].fold(Plus(0))
@alt(total)
fn total_slow(n: Int) -> Int = n + 5
let n: 1..=8 = read()
print(total(n))
