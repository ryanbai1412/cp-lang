# CCO 2018 P4 Gradient Descent (interactive): the hidden function is a sum of |x - xi| + |y - yi|,
# so it separates into two convex functions. Find each coordinate's minimum by binary search on the
# discrete slope, within the query budget.
let (n, m, k) = read (Int, Int, Int)
fn ask(x: Int, y: Int) -> Int = wire.ask("? {x} {y}", read Int)
let x = (1..=n).argmin_convex(i => ask(i, 1))
let y = (1..=m).argmin_convex(j => ask(x, j))
wire.answer("! {ask(x, y)}")
