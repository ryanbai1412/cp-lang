// expect: ok, output "YES"
var c = Counter.of([1, 2])
c.clear()
print(yes(c.len == 0 and c[1] == 0))
