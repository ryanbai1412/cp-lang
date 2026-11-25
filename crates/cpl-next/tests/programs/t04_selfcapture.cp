// expect: E0000
// (pending) a diagnostic code for: a function value captures a function value that can hold itself
// input: "3\n"
fn inc(x: Int) -> Int = x + 1
let n: Int = read()
var f: Int -> Int = inc
for i in 0..n:
    let g = f
    f = x => g(x) + i
print(f(0))
