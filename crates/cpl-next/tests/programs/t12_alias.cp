// expect: ok, output "6 6"
// input: "\n"
// A list passed by value while the callee pushes to the same list.
// Value semantics say `xs` does not change. Nothing in the slice enforces it.
var log: List<Int> = [1, 2, 3]
fn total(xs: List<Int>) -> Int:
    var s = 0
    for i in 0..xs.len:
        if log.len < 2000: log.push(xs[i])
        s += xs[i]
    s
print("{total(log)} {log.len}")
