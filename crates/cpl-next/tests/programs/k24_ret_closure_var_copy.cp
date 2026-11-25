// expect: ok, output "6"
// A var that is never written after the closure is made: spec still says by reference -> error on return?
fn adder(n: Int) -> Int -> Int:
    var k = n
    k += 1
    let kk = k
    x => x + kk
let f = adder(4)
print(f(1))
