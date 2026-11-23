// expect: exit 3
// rule: 06.4.3
// tool: judge {case} {dir}/in {dir}/out
// file: in "5\n"
// file: out "4\n"
// has: "assertion failed"
// A `@check` reads the input with `read()` and the output with `read_output()`; a failing `assert` rejects (decision 375).
let n: Int = read()
print(n - 1)

@check
fn ok():
    let n: Int = read()
    let out: Int = read_output()
    assert out == n
