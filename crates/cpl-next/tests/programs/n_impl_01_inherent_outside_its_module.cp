// expect: E0501
// rule: R08.1.3
// Inherent methods live in their type's module: a user file cannot add them to a library type.
impl List<Int>: // <- here
    fn second(self) -> Int = self[1]
print([1, 2].len)
