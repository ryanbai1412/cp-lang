// expect: ok, output "YES"
var c = Counter.of([1])
let d = c.take()
print(yes(c.is_empty and d[1] == 1))
