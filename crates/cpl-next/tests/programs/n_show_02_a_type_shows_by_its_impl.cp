// expect: ok, output "P(1, 2)\nat P(1, 2)\nP(1, 2) P(3, 4)"
// rule: R10.3.5
// A struct prints by its own `Show` impl, which may give only `show`.
struct P:
    a: Int
    b: Int
impl Show for P:
    fn show(self) -> Str = "P({a}, {b})"
let p = P(1, 2)
print(p)
print("at {p}")
print([p, P(3, 4)])
