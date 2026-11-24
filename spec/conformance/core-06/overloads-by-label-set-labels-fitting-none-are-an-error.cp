// expect: E0310
// rule: 06.1.6
// No `pick` takes both `by:` and `cmp:`, so the call fits none (R06.1.6).
fn pick(xs: List<Int>; by: Int -> Int) -> Int = 1
fn pick(xs: List<Int>; cmp: (Int, Int) -> Bool) -> Int = 2
print(pick([1], by: x => x, cmp: (a, b) => a < b)) // <- here
