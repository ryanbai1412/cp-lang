// expect: ok
// rule: 06.4.3
// tool: judge {case} {dir}/in {dir}/out
// file: in "1\n6 3\n"
// file: out "1\n"
// Under `@!multitest` a checker reads its test's input in its own shape, not the solution's (rebuild 28 F02).
@check
fn ok() -> Bool:
    let (n, x) = read<(Int, Int)>()
    n == 6 and x == 3

@!multitest
let n: Int = read()
let x: Int = read()
print(1)
