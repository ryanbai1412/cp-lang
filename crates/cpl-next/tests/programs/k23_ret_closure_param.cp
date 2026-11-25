// expect: ok, output "5"
// A closure over a by-value Int parameter (not a var) may be returned.
fn adder(n: Int) -> Int -> Int:
    x => x + n
let f = adder(4)
print(f(1))
