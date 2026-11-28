// expect: ok, output "500000500000"
// rule: 01.6.8
// A `const` worked out in under 10000000 steps is a constant: a million jumps back to the
// loop's start, and one fn entered, are within the budget.
fn tri(n: Int) -> Int:
    var s = 0
    for i in 1..=n:
        s += i
    s
const T = tri(1000000)
print(T)
