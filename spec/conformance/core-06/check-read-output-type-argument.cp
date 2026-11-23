// expect: ok
// rule: 06.4.3
// tool: judge {case} {dir}/in {dir}/out
// file: in "5\n"
// file: out "5\n"
// `read_output` takes a type argument like `read` (rebuild round 21 A7).
let n: Int = read()
print(n)

@check
fn ok() -> Bool:
    let n: Int = read()
    let out = read_output<Int>()
    out == n
