// expect: E0603
// (spec) R05.7.8 (verify)
fn mk(n: Int) -> (Int, Int -> Int):
    var k = n
    k += 1
    (1, x => x + k)  // <- here
let (a, f) = mk(5)
print(f(1))
