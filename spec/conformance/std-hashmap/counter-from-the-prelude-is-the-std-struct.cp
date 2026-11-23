// expect: ok, output "2 0 3"
// rule: std.hashmap.Counter
var c: Counter<Char, Int> = Counter(0)
for x in "abca": c.add(x, 1)
print(c['a'], c['z'], c.len)
