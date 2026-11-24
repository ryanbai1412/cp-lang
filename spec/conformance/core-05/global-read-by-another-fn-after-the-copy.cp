// expect: ok, output "1 2 9 3"
// rule: 05.3.2
// A global copied by one fn and read later by another keeps its value (adversarial round 47).
// copies: 1
let base = [1, 2]
fn build() -> List<Int>:
    var xs = base
    xs.push(9)
    xs
fn total() -> Int = base.sum()
let b = build()
print(b, total())
