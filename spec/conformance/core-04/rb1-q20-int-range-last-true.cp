// expect: ok, output "7"
// rule: 04.4.19
print((0..=10).last_true(k => k * k <= 50) ?? -1)
