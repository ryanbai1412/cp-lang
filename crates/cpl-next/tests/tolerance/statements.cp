// A broken top-level statement leaves what it declares declared.
domain V = 0..3
var a: [V]Int // injected
print(a[V(0)])
let n = 5
print(n * 2 + ) // injected
print(n)
fn twice(x: Int) -> Int:
    x * 2
print(twice(n))
