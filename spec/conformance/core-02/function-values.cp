// expect: ok, output "16 7"
// rule: 02.1.3
fn sq(x: Int) -> Int = x * x
fn ap(f: Int -> Int, x: Int) -> Int = f(x)
let k = 3
print(ap(sq, 4), ap(x => x + k, 4))
