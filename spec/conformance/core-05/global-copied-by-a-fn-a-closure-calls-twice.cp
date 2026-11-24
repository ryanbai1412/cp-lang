// expect: ok, output "8"
// rule: 05.3.2
// A fn copying a global, called once in the source from a closure that runs twice, copies it each time (adversarial round 47).
// copies: 1
let base = [1, 2, 3]
fn grow() -> Int:
    var xs = base
    xs.push(4)
    xs.len
let twice = () => grow() + grow()
print(twice())
