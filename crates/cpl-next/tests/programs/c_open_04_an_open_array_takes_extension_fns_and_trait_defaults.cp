// expect: ok, output "7\nYES"
// rule: R04.8.11
// An open `[D]T` reaches a `Seq` extension fn (`max`) and an `Indexed` default (`cut_after`) as any receiver does.
fn top<D: Discrete>(a: [D]Int) -> Int = a.max() ?? 0
fn after<D: Ordered, T>(a: [D]T, i: D) -> Cut<D> = a.cut_after(i)
domain V = 0..3
let v: [V]Int = [7, 1, 3]
print(top(v))
print(yes(after(v, V(2)) == V(2).r))
