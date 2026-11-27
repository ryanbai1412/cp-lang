// expect: ok, output "1 2 1\n7"
// rule: R08.1.4
// Two traits that both give `S` an `m` make `s.m()` ambiguous; `A.m(s)` and `B.m(s)` name the
// impl, and a default reached through `B` calls `B`'s `m`.
trait A:
    fn m(self) -> Int
trait B:
    fn m(self) -> Int
    fn add(self, k: Int) -> Int = self.m() + k
struct S: x: Int
impl A for S:
    fn m(self) -> Int = 1
impl B for S:
    fn m(self) -> Int = 2
let s = S(0)
print(A.m(s), B.m(s), A.m(S(3)))
print(B.add(s, 5))
