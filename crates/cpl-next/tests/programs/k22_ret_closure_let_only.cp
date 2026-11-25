// expect: ok, output "9"
// A closure over `let`s only may be returned (false-rejection probe).
fn adder(n: Int) -> Int -> Int:
    let k = n * 2
    x => x + k
print(adder(4)(1))
