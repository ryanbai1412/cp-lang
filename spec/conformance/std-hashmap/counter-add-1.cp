// expect: ok, output "YES"
var c: Counter<Char, Int> = Counter(0)
c.add('x', 2)
print(yes(c['x'] == 2))
