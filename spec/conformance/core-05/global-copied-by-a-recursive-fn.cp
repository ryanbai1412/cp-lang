// expect: ok, output "9"
// rule: 05.3.2
// A recursive fn copies a global into its `var` on every call; one call site is not one run (adversarial round 47).
let base = [5, 6]
fn take(d: Int) -> Int:
    var xs = base
    xs.push(d)
    if d == 0: xs.len else: xs.len + take(d - 1)
print(take(2))
