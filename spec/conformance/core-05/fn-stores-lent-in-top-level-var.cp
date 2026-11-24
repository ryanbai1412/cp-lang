// expect: E0603
// rule: 05.7.10
var kept = List<Int>()
fn keep(xs: List<Int>):
    kept = xs // <- here
keep([1, 2])
print(kept.len)
