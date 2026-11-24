// expect: E0701
// rule: 08.1.4
// has: "`T.foo` is the type's own member"
// A fn `foo(self: T)` beside `T`'s own member `foo` is dead in the module that declares it: the call reaches the member.
struct T(k: Int):
    pub fn foo(self) -> Int = self.k
fn foo(self: T) -> Int = 0 - self.k
print(T(3).foo()) // <- here
