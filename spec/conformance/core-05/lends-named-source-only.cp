// expect: E0603 at 1
// rule: 05.6.1
fn pick(a: List<Int>, b: List<Int>) -> lends(a) List<Int> = b
print(pick([1], [2]).len)
