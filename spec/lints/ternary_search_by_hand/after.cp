let n: 1..=100000 = read()
let a: [0..n]Int = read()
fn cost(x: Int) -> Int = sum(abs(v - x) for v in a)
let best = ternary_min(-1_000_000_000..1_000_000_001, cost)!
print(cost(best))
