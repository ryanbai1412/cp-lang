// expect: exit 2
// rule: 10.1.3
// tool: run {case}
// has: "--grader"
// `cpl run` of an entry `@grader fn` with no grader.cpp found is an error naming `--grader`, not a silent exit 0 (decision 136).
extern fn compare(a: Int, b: Int) -> Int
@grader
fn solve(n: Int) -> Int = compare(1, n)
