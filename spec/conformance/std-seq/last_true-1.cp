// expect: ok, output "YES"
print(yes((0..100).last_true(x => x * x <= 50) == Some(7)))
