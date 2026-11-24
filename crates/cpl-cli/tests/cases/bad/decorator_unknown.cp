// error: unknown decorator `@memoize`
@memoize
fn f(n: 0..=10) -> Int = n
print(f(3))
