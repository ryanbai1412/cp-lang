// expect: ok, output "8 5"
// rule: 08.1.3
struct P:
    x: Int
    fn dbl(self) -> Int = self.x * 2
impl P:
    fn inc(self) -> Int = self.x + 1
print(P(4).dbl(), P(4).inc())
