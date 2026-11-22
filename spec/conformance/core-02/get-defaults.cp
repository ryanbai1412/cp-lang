// expect: ok, output "5 7"
// rule: 02.6.6
struct P:
    x: Int
    @get
    fn shifted(self, by: Int = 1) -> Int = self.x + by
let p = P(x: 4)
print(p.shifted, p.shifted(by: 3))
