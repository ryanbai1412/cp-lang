// expect: ok, output "7 12"
// rule: 05.10.1
// A parameter with no heap part is copied in like a plain `let` (decision 471), so a closure capturing it can be returned.
fn adder(n: Int) -> (Int -> Int):
    (k: Int) => k * n + 1
fn twice(n: Int) -> (Int -> Int):
    let m = n * 2
    (k: Int) => k * m
print(adder(3)(2), twice(3)(2))
