// expect: E0606
// rule: R08.1.4
// `Trait.m(x)` names `x`'s impl of the trait, so a type with no impl of it has no `m` there.
trait A:
    fn m(self) -> Int
struct S: x: Int
struct T: x: Int
impl A for S:
    fn m(self) -> Int = 1
print(A.m(T(0))) // <- here
