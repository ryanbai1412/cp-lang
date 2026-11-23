// expect: exit 3
// rule: 06.4.3
// tool: judge {case} {dir}/in {dir}/out
// file: in "5\n"
// file: out "4\n"
// A `@check fn ok() -> Bool` that returns `false` rejects the output, like a failing `assert` (decision 375).
let n: Int = read()
print(n - 1)

@check
fn ok() -> Bool:
    let n: Int = read()
    let out: Int = read_output()
    out == n
