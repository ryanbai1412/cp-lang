// expect: E0603
// (spec) R05.7.9 (verify)
// Escape through a `mut` parameter (the docs admit this one is unchecked).
fn mk(mut out: List<Int -> Int>, n: Int):
    var k = n
    out.push(x => x + k)  // <- here
fn noise(a: Int) -> Int:
    var t = a * 1000
    t += 1
    t
var fs: List<Int -> Int> = []
mk(mut fs, 5)
print(noise(3))
print(fs[0](1))
