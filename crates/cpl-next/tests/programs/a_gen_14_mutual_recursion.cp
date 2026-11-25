// expect: ok, output "1"
// rule: R06
// Mutual recursion between two top-level fns declared in either order.
fn even(n: Int) -> Bool = if n == 0: true else: odd(n - 1)
fn odd(n: Int) -> Bool = if n == 0: false else: even(n - 1)
print(if even(10): 1 else: 0)
