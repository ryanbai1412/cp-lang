// expect: E0603
// rule: 05.5.1
fn same(xs: List<Int>) -> List<Int> = xs // <- here
print(same([1]))
