// expect: ok, output "1 2"
// rule: 07.12.5
let a = [1, 3, 3, 5]
print(Int(a.bsearch(x => x >= 3)), a.equal_range(3).len)
