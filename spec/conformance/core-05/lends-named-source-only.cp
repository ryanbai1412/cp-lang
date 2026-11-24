// expect: E0603
// rule: 05.6.1
fn pick(a: List<Int>, b: List<Int>) -> lends(a) List<Int> = b // <- here
print(pick([1], [2]).len)
