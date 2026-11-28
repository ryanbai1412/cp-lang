// expect: ok, output "1 2"
// engines: next
// rule: 04.2.31
// R04.2.31: `Ordered` is stepping and order, so a generic body that steps bounds `D: Ordered`.
fn second<D: Ordered>(d: D) -> ?D = d.next
domain V = 0..3
print(second(V(0))!.ord, second(V(1))!.ord)
