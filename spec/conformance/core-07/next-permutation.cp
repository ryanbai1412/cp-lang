// expect: ok, output "6"
// rule: 07.12.6
var p = [1, 2, 3]
var c = 1
while p.next_permutation(): c += 1
print(c)
