// expect: ok, output "2"
var c: Counter<Char, Int> = Counter()
c.push('a')
c.push('a')
print(c['a'])
