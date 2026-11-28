// expect: ok, output "YES NO"
// engines: next
let k = [5, 6, 7, 8, 9, 10, 11].step(3).keys
print(yes(k.contains(3)), yes(k.contains(4)))
