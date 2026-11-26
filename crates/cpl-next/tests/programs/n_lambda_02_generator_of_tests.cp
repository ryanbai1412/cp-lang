// expect: ok, output "2 false true 1"
// rule: R01.10.1
// A generator of tests passed where the fn takes a test is that test: `count(t for x in xs)` counts the trues.
let xs = [1, 2, 3, 4]
print(count(x % 2 == 0 for x in xs), all(x > 1 for x in xs), any(x > 3 for x in xs), count((a, b) == (2, 1) for (a, b) in [(1, 2), (2, 1)]))
