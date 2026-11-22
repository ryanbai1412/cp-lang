// expect: ok, output "5 2"
// rule: 05.8.3
struct P:
    var xs: List<Int>
    pub fn lo(mut self) -> lends mut Int = self.xs[0]
var p = P(xs: [1, 2])
p.lo() = 5
print(p.xs[0], p.xs[1])
