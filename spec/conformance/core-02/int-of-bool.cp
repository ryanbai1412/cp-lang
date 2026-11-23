// expect: ok, output "1 0 2"
// rule: 02.2.1
// `Int(b)` turns a `Bool` into 1 or 0, so flags count without a lambda (rebuild 8, H-BOOL-TO-INT).
print(Int(true), Int(false), [3, 8, 5].map(x => Int(x > 4)).sum())
