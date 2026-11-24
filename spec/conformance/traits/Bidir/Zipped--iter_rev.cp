// expect: ok, output "2 5\n1 4"
for (x, y) in zip([1, 2, 3], [4, 5]).rev: print(x, y)
