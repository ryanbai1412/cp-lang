// expect: E0701
// rule: 08.1.4
// Two extension functions of one name that both take the receiver are ambiguous; a type's own cost is a trait override (decision 486).
pub fn sum(self: Range<Int>) -> Int = 7
print((0..3).sum()) // <- here
