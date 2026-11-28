// expect: E0606
// rule: R04.8.10
// A list becomes `[D]T` only where `D` counts its positions: the coercion's `D: Domain` is checked
// at the call, never left to fail when the program is linked.
fn g<D: Discrete, T>(a: [D]T, d: D) -> Int = d.ord
let k = 2
print(g([4, 5, 6], k))
