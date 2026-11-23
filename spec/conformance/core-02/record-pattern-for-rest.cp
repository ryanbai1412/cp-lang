// expect: ok, output "1 5"
// rule: 2.7.6
let es = [(to: 1, w: 2), (to: 5, w: 6)]
var seen = List<Int>()
for (to: u, ..) in es: seen.push(u)
print(seen)
