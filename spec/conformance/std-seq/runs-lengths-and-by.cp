// expect: ok, output "2 1 3\n2 3 1"
// rule: 07.7.10
print([r.len for r in "aabccc".runs()])
print([r.len for r in [1, 3, 2, 4, 6, 5].runs(by: x => x % 2)])
