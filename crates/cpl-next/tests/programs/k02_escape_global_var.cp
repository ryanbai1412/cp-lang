// expect: E0603
// (spec) R05.7.10
// Same, by assignment to a top-level function variable.
var f = (x: Int) => x
fn mk(n: Int):
    var k = n
    f = x => x + k  // <- here
fn noise(a: Int) -> Int:
    var t = a * 1000
    t += 1
    t
mk(5)
print(noise(3))
print(f(1))
