// expect: E0501
// rule: 08.1.3
// Inherent methods live in their type's module, so a user file can't add them to a built-in type; before, the block was accepted and never called.
impl Range<Int>: // <- here
    fn count(self) -> Int = 42
print((1..10).count())
