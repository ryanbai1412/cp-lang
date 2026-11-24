// expect: ok, output "YES"
var c: Counter<Char, Int> = Counter()
c.add('x', 2)
c.add('x', -2)
print(yes(c.len == 0))
