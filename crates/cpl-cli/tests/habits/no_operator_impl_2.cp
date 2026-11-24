struct P(x: Int)
impl Add<Int> for P:
    fn add(self, o: Int) -> P = P(x + o)
fn one() -> Int = 1
let a = P(1)
print((a + one).x)  // <- here
