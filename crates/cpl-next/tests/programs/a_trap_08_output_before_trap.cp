// expect: R0401
// rule: diagnostics R0401; output "1" is kept
// Output before a trap is kept.
print(1)
let xs = [1]
assert xs.len == 2  // <- here
print(2)
