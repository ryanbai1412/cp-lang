// expect: ok
// rule: trace.5
// tool: dap break:3 eval:acc reverseContinue eval:acc
// has: "\"reason\":\"breakpoint\""
// has: "\"result\":\"1\""
// has: "\"result\":\"0\""
// `cpl dap` stops at a breakpoint, evaluates a variable there, and runs backward to the breakpoint's earlier hit, where the variable has its older value.
var acc = 0
fn add(v: Int):
    acc += v
for v in 1..=3:
    add(v)
print(acc)
