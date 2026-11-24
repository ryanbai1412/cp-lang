// expect: ok, output "3\n4\n5"
let a = [3, 1, 4, 1, 5]
for x in a.filter(x => x > 1): print(x)
