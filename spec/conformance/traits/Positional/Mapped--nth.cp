// expect: ok, output "8"
let a = [3, 1, 4, 1, 5]
print(a.map(x => x * 2).nth(2))
