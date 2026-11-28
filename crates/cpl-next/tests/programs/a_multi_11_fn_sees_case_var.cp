// expect: ok, output "3\n3"
// rule: R10.1.1
// input: "2\n3\n4\n"
// R10.1.1: a fn reads and writes a module `var` below the divider; each case starts it afresh.
@!multitest
let n: 1..=9 = read()
var xs = [1, 2]
fn f():
    xs.push(n)
f()
print(xs.len)
