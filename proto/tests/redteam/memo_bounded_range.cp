// `@memo` over bounded integer ranges (including nonzero/negative lower bounds and dynamic upper bounds)
// lowers to a flat dense table with exact offset arithmetic.
let w: 1..=20 = read()
@memo
fn ways(x: -3..=3, y: 1..=w) -> Int:
    if y == 1: return x + 10
    ways(x, y - 1) + (if x > -3: ways(x - 1, y - 1) else: 0)
print(ways(3, w), ways(-3, w), ways(0, 1))
