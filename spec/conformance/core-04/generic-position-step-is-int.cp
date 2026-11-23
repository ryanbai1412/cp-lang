// expect: ok, output "3 2"
// rule: 04.8.2
// Position arithmetic on a `D: Domain` parameter gives an `Int`, with no `Add` or `Sub` bound.
domain V = 0..9
fn after<D: Domain>(p: D) -> Int = p + 1
fn gap<D: Domain>(a: D, b: D) -> Int = b - a
print(after(V(2)), gap(V(4), V(6)))
