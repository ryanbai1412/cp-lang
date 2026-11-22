let a: F64 = 0.1 + 0.2
print(yes(a < 0.5))
fn half(x: F64) -> F64 = x / 2.0
fn count(xs: List<Int>, v: Int) -> Int = xs.count(x => x == v)
print(count([1, 2, 2], 2))
fn exp(xs: List<Int>) -> List<Int> = [x * 2 for x in xs]
print(yes(exp([1]) == [2]))
