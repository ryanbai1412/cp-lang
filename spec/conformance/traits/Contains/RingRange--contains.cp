// expect: ok, output "YES NO"
// engines: next
let r = [1, 2, 3].ring.span(2, 2)
print(yes(r.contains(0)), yes(r.contains(1)))
