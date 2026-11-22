// expect: ok, output "3 4"
// rule: 07.3.4
let a = [1, 2, 3, 4]
print(a.filter(x => x % 2 == 1).rev.first!, a.last!)
