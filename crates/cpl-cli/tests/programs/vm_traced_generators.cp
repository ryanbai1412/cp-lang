// A traced run feeds every loop over a `yields` fn on the VM: a loop in
// a comprehension, a generator whose own loop runs eagerly while a loop
// is fed from it, a loop in a closure, and a yield of a call's value.
fn upto(n: Int) -> yields Int:
    for i in 0..n: yield i
fn sq(x: Int) -> Int = x * x
fn squares(n: Int) -> yields Int:
    for i in upto(n): yield sq(i)
var seen: List<Int> = []
for x in squares(4):
    seen.push(x)
print(seen)
print([x + 1 for x in upto(3)])
print([y for y in squares(3) if y > 0])
let doubled = (k: Int) => [x * 2 for x in upto(k)]
print(doubled(4))
