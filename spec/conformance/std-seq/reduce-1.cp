// expect: ok, output "YES"
print(yes([3, 4].reduce((a, b) => a * b) == Some(12)))
