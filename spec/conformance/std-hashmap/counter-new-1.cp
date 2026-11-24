// expect: ok, output "YES"
var c: Counter<Char, Int> = Counter()
print(yes(c.get('z') == 0 and c.len == 0))
