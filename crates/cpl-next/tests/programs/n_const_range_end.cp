// expect: ok, output "3 5\n4 5"
// rule: R03.1.4
// A range type outside a body may end at a `const` whose value is written as a literal: its
// bounds are that literal's, in a field, an alias, a parameter and a result.
mod lim:
    pub const CAP = 5
    type Count = 0..=CAP
    pub struct Box:
        pub n: 0..=CAP
    pub fn top() -> Count = CAP
    pub fn same(k: 0..=CAP) -> Int = k
use lim.Box
use lim.top
use lim.same
let b = Box(n: 3)
print(b.n, top())
print(same(4), same(top()))
