// expect: ok, output "4"
// rule: 02.7.3
enum Dir = L | R | U | D
let seen = [Dir] d => 1
print(seen.sum())
