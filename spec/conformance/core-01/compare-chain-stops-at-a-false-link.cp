// expect: ok, output "NO\nNO\nYES\n3\nNO\n2"
// rule: 01.5.3
// A chain keeps Python's meaning: each operand runs once, left to right, and a
// false link stops the chain before the next operand is read.
var calls = 0
fn side(x: Int) -> Int:
    calls += 1
    return x
let a: List<Int> = []
let n = 0
print(yes(0 < n <= a[n - 1]))
print(yes(0 < n <= a[n - 1] < a[n]))
calls = 0
print(yes(side(1) < side(2) < side(3)))
print(calls)
calls = 0
print(yes(side(2) < side(1) < side(3)))
print(calls)
