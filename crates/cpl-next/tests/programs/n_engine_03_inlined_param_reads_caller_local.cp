// expect: ok, output "30 4 1099"
// The engine gives a small fn's parameter the caller's own local when nothing writes that
// local during the call; a local a closure can write still gets its own copy.
fn twice(x: Int) -> Int = x + x
fn first(a: Int, f: () -> ()) -> Int:
    f()
    a
fn go() -> Int:
    var v = 1
    let w = () => v = 99
    first(v, w) * 1000 + v
var s = 0
var i = 0
while i < 4:
    s += twice(i) + i
    i += 1
var k = 3
k = twice(k) - twice(1)
print(s + 12, k, go())
