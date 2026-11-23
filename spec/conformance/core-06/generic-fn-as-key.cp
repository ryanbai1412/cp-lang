// expect: ok, output "3 2 1"
// rule: 06.1.8
// A generic `fn` stands in for a lambda: `by: desc` (rebuild 8, H-DESC-AS-VALUE).
print([3, 1, 2].sorted(by: desc))
