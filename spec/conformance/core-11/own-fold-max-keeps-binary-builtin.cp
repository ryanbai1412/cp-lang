// expect: ok, output "9 1"
// rule: 11.2.3 11.3.3
// A file's own fold `fn max(self: ..)` takes one value, so `max` of two
// values stays the core builtin; a call of one value is the fold.
fn max(self: [3]Int) -> Int = self[0]
print(max(4, 9), max([1, 2, 3]))
