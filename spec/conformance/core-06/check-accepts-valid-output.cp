// expect: ok
// rule: 06.4.3
// tool: judge {case} {dir}/in {dir}/out
// file: in "5\n"
// file: out "5\n"
// A `@check` that holds accepts: `cpl judge` exits 0.
let n: Int = read()
print(n - 1)

@check
fn ok() -> Bool:
    let n: Int = read()
    let out: Int = read_output()
    out == n
