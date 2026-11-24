// expect: ok, output "5"
fn count<S: Sized>(s: S) -> Int = s.len
print(count([3, 1, 4, 1, 5].rev))
