// expect: ok, output "8"
// rule: 04.4.1
// In expression position `0..=1e9` is a range value, not a type.
print((0..=1e9).first_true(x => x * x >= 50)!)
