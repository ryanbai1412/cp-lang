// expect: ok, output "2 2 5"
// rule: 01.10.1
let xs = [1, 2, 3, 4, 5]
let bs = [x % 2 == 0 for x in xs]
assert not all(x > 1 for x in xs)
assert any(x > 4 for x in xs)
print(count(x % 2 == 0 for x in xs), count(b for b in bs), bs.count())
