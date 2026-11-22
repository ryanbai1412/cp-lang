// expect: ok, output "3"
// rule: 06.4.2
struct P:
    x: Int
    y: Int
    @get
    pub fn total(self) -> Int = self.x + self.y
let p = P(x: 1, y: 2)
print(p.total)
