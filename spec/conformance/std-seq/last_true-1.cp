// expect: ok, output "YES"
print((0..100).last_true(x => x * x <= 50) == Some(7))
