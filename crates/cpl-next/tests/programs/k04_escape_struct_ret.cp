// expect: E0603
// (spec) R05.7.8 (verify)
// Escape inside a struct that is returned.
struct Box:
    f: Int -> Int
    tag: Int
fn mk(n: Int) -> Box:
    var k = n
    k += 1
    Box(x => x + k, 1)  // <- here
fn noise(a: Int) -> Int:
    var t = a * 1000
    t += 1
    t
let b = mk(5)
print(noise(3))
print(b.f(1))
