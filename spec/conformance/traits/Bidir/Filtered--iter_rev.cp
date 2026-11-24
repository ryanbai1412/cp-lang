// expect: ok, output "5\n4\n3"
let a = [3, 1, 4, 1, 5]
for x in a.filter(x => x > 1).rev: print(x)
