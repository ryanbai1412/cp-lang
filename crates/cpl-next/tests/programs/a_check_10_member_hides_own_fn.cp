// expect: E0701
// rule: 08.1.4
// A fn beside a type's own member of its name, with that type as `self`, is never called.
struct T(k: Int):
    pub fn foo(self) -> Int = self.k
fn foo(self: T) -> Int = 0 - self.k
print(T(3).foo())  // <- here
