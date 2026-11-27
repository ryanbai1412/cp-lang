// expect: E0503
// rule: 06.4.11
// `@was(total)` on a member: the old name is a removed spelling naming the new one.
struct Bag(xs: List<Int>):
    @was(total)
    pub fn weight(self) -> Int = self.xs.len
print(Bag([1, 2]).total())  // <- here
