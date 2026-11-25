// expect: ok, output "0 2 4"
// rule: R06
// `continue` in a plain loop skips the rest of the body and runs the step.
var out: List<Int> = []
for i in 0..6:
    if i % 2 == 1: continue
    out.push(i)
print(out)
