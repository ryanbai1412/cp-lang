// expect: ok, output "7 15 12"
// rule: 11.2.3 11.3.3
// A top-level or local closure named `max` or `min` wins over the core builtins.
let max = (a: Int, b: Int) => a + b
fn f(x: Int) -> Int:
    let min = (a: Int, b: Int) => a * b
    return min(x, 3)
print(max(4, 3), f(5), [3, 12].max()!)
