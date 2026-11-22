// expect: ok
// rule: 10.1.3
extern fn compare(a: Int, b: Int) -> Int
@grader
fn solve(n: Int) -> Int = compare(1, n)
