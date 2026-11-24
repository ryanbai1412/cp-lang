struct P(x: Int)
impl Add<P> for P:
    fn add(self, o: P) -> P = P(x + o.x)
let a = P(1)
print((a + 2).x)  // <- here
