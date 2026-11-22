// expect: ok, output "0 2 4"
// rule: 07.2.10
fn evens(n: Int) -> yields Int = [i * 2 for i in 0..n]
print(evens(3).collect())
