// A lowercase generic fn takes a `const` argument written after its name, as a type does.
fn shift<const K: Int>(x: Int) -> Int = x * K - K
fn pow_of<const B: Int>(e: Int) -> Int = B ** e
let k = 4
print(shift<3>(5), shift<2 + 1>(k), pow_of<10>(3))
