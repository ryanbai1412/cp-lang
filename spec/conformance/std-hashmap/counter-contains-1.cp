// expect: ok, output "YES"
let c = Counter.of([5])
print(yes(c.contains(5) and not c.contains(6)))
