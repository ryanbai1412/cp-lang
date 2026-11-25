// expect: ok, output "112"
// Closure sharing a var, called from a generator body and after it.
fn evens(n: Int) -> yields Int:
    for i in 0..n:
        if i % 2 == 0: yield i
fn go(n: Int) -> Int:
    var s = 0
    let add = (x: Int) => s += x
    for e in evens(n): add(e)
    add(100)
    s
print(go(7))
