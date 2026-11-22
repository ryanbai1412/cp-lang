// expect: ok, output "0"
// rule: 04.2.1 04.8.4
let m = 0
domain E = 0..m
let deg = [E] e => 1
print(deg.sum() + E.count)
