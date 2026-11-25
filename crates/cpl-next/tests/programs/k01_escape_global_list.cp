// expect: E0603
// (spec) R05.7.10 (judged by name, verify)
// A closure sharing a local `var` is stored in a top-level list by push, then called after the fn returned.
var fs: List<Int -> Int> = []
fn mk(n: Int):
    var k = n
    fs.push(x => x + k)  // <- here
    k += 1
fn noise(a: Int, b: Int) -> Int:
    var t = a * 1000 + b
    t += 1
    t
mk(5)
mk(7)
print(noise(1, 2))
print(fs[0](1), fs[1](1))
