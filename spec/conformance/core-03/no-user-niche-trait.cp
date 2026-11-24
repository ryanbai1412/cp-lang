// expect: E0204
// rule: 03.2.3
struct P(a: Int)
impl Niche for P // <- here
print(1)
