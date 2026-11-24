// expect: exit 1
// rule: trace.3
// before: run --trace
// tool: replay {case} --trace {dir}/replay-at-a-step-with-no-call-is-an-error.trace.html --step 1
// has: "no fn call is running at step 1"
// A step of the recording where no fn call is open has nothing to replay, so `cpl replay` says so instead of rerunning the program.
var acc = 0
fn add(v: Int):
    acc += v
for v in 1..=3:
    add(v)
print(acc)
