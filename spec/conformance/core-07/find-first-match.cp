// expect: ok, output "4"
// rule: 07.9.5
// `xs.find(p)` is the first element where `p` holds, as `?T` (rebuild 7, H-FIND-FIRST).
print([3, 4, 5].find(x => x > 3)!)
