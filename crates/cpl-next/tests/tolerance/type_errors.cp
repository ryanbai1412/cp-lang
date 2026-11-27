// Check errors in a fn body and in top-level statements stop nothing else.
struct P:
    x: Int
    y: Int

fn norm(p: P) -> Int:
    p.x * p.x + p.y * p.y

fn typo(a: Int) -> Int: // injected
    a + zz

fn wrong_arg() -> Int: // injected
    norm(3)

let p = P(x: 1, y: 2)
let s: Int = "s" // injected
print(norm(p))
print(s + 1)
var total = 0
for i in 0..10:
    total += i
print(total)
