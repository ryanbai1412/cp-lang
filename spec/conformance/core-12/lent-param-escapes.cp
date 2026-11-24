// expect: E0603
// rule: 12.4.8 12.4.10
fn keep(xs: List<Int>) -> List<Int> = xs // <- here
print(keep([1]))
