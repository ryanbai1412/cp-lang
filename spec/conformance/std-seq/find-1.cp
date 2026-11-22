// expect: ok, output "YES"
print((1..=10).find(x => x * x > 20) == Some(5))
