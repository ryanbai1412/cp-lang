// expect: ok, output "YES"
print([1, 2, 3, 4].filter(x => x % 2 == 0).collect() == [2, 4])
