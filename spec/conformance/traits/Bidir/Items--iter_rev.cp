// expect: ok, output "4 5\n3 1\n2 4\n1 1\n0 3"
let a = [3, 1, 4, 1, 5]
for (k, x) in a.items.rev: print(k, x)
