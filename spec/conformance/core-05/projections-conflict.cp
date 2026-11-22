// expect: E0601 at 6
// rule: 05.8.3
struct P:
    var xs: List<Int>
    pub fn lo(mut self) -> lends mut Int = self.xs[0]
    pub fn hi(mut self) -> lends mut Int = self.xs[1]
var p = P(xs: [1, 2])
swap(mut p.lo(), mut p.hi())
