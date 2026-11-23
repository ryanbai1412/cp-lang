// expect: ok, output "4 16 36"
// rule: 07.8.1
// A comprehension takes an `if` filter (rebuild 7, H-COMPREHENSION-FILTER).
print([x * x for x in 1..=6 if x % 2 == 0])
