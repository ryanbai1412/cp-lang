// expect: ok, output "1"
// rule: 06.1.6
// A `by:` fn returning `Bool` is still the `by:` overload: labels pick, not types (R06.1.6).
fn pick(xs: List<Int>; by: Int -> Bool) -> Int = 1
fn pick(xs: List<Int>; cmp: (Int, Int) -> Bool) -> Int = 2
print(pick([1], by: x => x > 0))
