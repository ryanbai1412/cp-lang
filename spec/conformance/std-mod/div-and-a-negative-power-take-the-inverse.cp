// expect: ok, output "2 1 5"
type M = Mod<7>
fn ratio(a: M, b: M) -> M = a / b
print(M(3) / M(5), M(2) ** -3, ratio(M(1), M(3)))
