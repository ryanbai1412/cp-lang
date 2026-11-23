// expect: ok, output "4"
// rule: 08.1.1
// An associated type may carry bounds (`type Arc: Target`), and a trait alias may bound it (`where Self.Arc: ...`).
trait Target:
    fn to(self) -> Int
trait Next:
    type Arc: Target
    fn first(self) -> Self.Arc
trait Labelled:
    fn label(self) -> Int
trait LabelledNext = Next where Self.Arc: Labelled
struct A(v: Int)
impl Target for A:
    fn to(self) -> Int = self.v
struct G(k: Int)
impl Next for G:
    type Arc = A
    fn first(self) -> A = A(self.k + 1)
print(G(3).first().to())
