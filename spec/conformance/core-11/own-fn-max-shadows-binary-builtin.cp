// expect: ok, output "7 2"
// rule: 11.2.3 11.3.3
// The file's own two-argument `max` and `min` win over the core builtins.
fn max(a: Int, b: Int) -> Int = a + b
fn min(a: Int, b: Int) -> Int = a - b
print(max(4, 3), min(5, 3))
