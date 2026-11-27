// expect: ok, output "0"
fn fresh<T: Default>(x: T) -> T = T.default()
print(fresh("ab").len)
