// expect: ok, output "300000"
// rule: 13.1.2
fn depth(n: Int) -> Int = if n == 0: 0 else: 1 + depth(n - 1)
print(depth(300000))
