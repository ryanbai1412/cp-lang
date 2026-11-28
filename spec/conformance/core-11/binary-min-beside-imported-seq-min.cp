// expect: ok, output "1 9 2"
// rule: 11.3.3
// Two values call the binary `min`/`max`; a sequence's fold is its `Seq` method, so the two never meet.
print(min(3, 1), max(4, 9), [5, 2].min()!)
