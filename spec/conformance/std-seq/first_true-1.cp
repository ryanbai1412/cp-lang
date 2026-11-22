// expect: ok, output "YES"
print((0..100).first_true(x => x * x >= 50) == Some(8))
