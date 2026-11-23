// expect: ok, output "3 1 3 4 4"
// rule: 07.1.7
// O(1) things are getters (no parens); allocating ones are named calls.
let xs = [4, 1, 3]
print(xs.len, xs.sorted(), xs.first)
