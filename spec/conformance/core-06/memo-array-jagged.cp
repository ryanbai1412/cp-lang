// expect: ok, output "155117520"
// rule: 06.4.1
@memo(array)
fn c(n: 0..=30, k: 0..=n) -> Int = if k == 0 or k == n: 1 else: c(n - 1, k - 1) + c(n - 1, k)
print(c(30, 15))
