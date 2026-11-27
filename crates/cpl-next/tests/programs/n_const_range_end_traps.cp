// expect: R0201
// rule: R03.1.4
// A field ending at a `const` checks a run-time value against that const.
mod lim:
    pub const CAP = 5
    pub struct Box:
        pub n: 0..=CAP
use lim.Box
var k = 5
print(Box(n: k).n)
k += 1
print(Box(n: k).n)  // <- here
