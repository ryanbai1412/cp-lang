// expect: ok, output "YES"
let c = Counter.of([5, 5])
print(yes(c[5] == 2 and c[6] == 0 and c.len == 1))
