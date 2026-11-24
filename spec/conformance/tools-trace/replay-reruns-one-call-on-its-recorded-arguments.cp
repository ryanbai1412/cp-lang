// expect: ok
// rule: trace.3
// before: run --trace
// tool: replay {case} --trace {dir}/replay-reruns-one-call-on-its-recorded-arguments.trace.html --step 3
// has: "replayed call 2 of `add(v: 2)`"
// has: "writes acc = 3 (as before)"
// `cpl replay` reruns the call open at a step of a recorded run on the arguments it had then, without rerunning the rest of the program.
var acc = 0
fn add(v: Int):
    acc += v
for v in 1..=3:
    add(v)
print(acc)
