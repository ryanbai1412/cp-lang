// expect: E0203
// rule: 06.4.11
// `@aka(size)` on a member: a call by that name says the member's own name.
struct Bag(xs: List<Int>):
    @aka(size)
    pub fn count(self) -> Int = self.xs.len
print(Bag([1, 2]).size())  // <- here
