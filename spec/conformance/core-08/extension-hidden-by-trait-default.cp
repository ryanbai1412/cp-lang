// expect: E0701
// rule: 08.1.4
// engines: next
// has: "`Range.sum` is the type's own member"
// has: "`(0..3).sum` never calls"
// A concrete `self: Range<Int>` loses to `Seq.sum`, a member of every `Seq`: a closed-form sum is the type's `impl Seq` override, not an extension fn.
pub fn sum(self: Range<Int>) -> Int = 7 // <- here
print((0..3).sum())
